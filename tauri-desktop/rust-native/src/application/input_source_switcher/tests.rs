use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

use super::{InputSourceSwitcher, SwitchDecision};
use crate::{
    application::{
        ports::{
            clock::Clock,
            diagnostic_log::{DiagnosticLogSink, SharedDiagnosticLogSink},
            input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
            settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
            switcher_events::SwitcherEventPublisher,
        },
        settings::SwitcherSettings,
        switcher_state::{SwitchLogEntry, SwitcherState},
    },
    domain::input_source::{
        FocusRule, FocusSnapshot, InputSourceId, LanguageTag, Platform, PlatformAppId, RuleTarget,
    },
};

const APP_ID: &str = "com.example.Editor";

#[derive(Default)]
struct GatewayData {
    current: String,
    selected: Vec<String>,
    select_attempts: usize,
    refreshes: usize,
    fail_first_select: bool,
}

struct FakeGateway(Arc<Mutex<GatewayData>>);

impl InputSourceGateway for FakeGateway {
    fn list(
        &mut self,
        refresh: bool,
        _locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError> {
        if refresh {
            self.0.lock().expect("gateway lock").refreshes += 1;
        }
        Ok(Vec::new())
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        InputSourceId::parse(&self.0.lock().expect("gateway lock").current)
            .map_err(|error| GatewayError::Operation(error.to_string()))
    }

    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError> {
        let mut data = self.0.lock().expect("gateway lock");
        data.select_attempts += 1;
        if data.fail_first_select && data.select_attempts == 1 {
            return Err(GatewayError::Operation("stale cache".to_owned()));
        }
        data.current = id.as_str().to_owned();
        data.selected.push(id.as_str().to_owned());
        Ok(())
    }
}

struct FakeRepository {
    fail_save: Arc<AtomicBool>,
}

impl SettingsRepository for FakeRepository {
    fn load(&self) -> Result<LoadOutcome, SettingsError> {
        Ok(LoadOutcome::Defaults {
            settings: SwitcherSettings::defaults(),
        })
    }

    fn save(&self, _settings: &SwitcherSettings) -> Result<(), SettingsError> {
        if self.fail_save.load(Ordering::Relaxed) {
            Err(SettingsError::Io("disk full".to_owned()))
        } else {
            Ok(())
        }
    }
}

struct FakeClock(Arc<AtomicU64>);

impl Clock for FakeClock {
    fn now_unix_ms(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

struct FakePublisher(Arc<Mutex<Vec<u64>>>);

impl SwitcherEventPublisher for FakePublisher {
    fn publish(&self, state: &SwitcherState, _locale: &str) {
        self.0.lock().expect("events lock").push(state.version);
    }
}

struct Fixture {
    switcher: InputSourceSwitcher<FakeGateway, FakeRepository, FakeClock, FakePublisher>,
    gateway: Arc<Mutex<GatewayData>>,
    fail_save: Arc<AtomicBool>,
    now: Arc<AtomicU64>,
    versions: Arc<Mutex<Vec<u64>>>,
}

impl Fixture {
    fn new(mut settings: SwitcherSettings) -> Self {
        settings.rules = vec![FocusRule::new(
            "editor",
            "Editor",
            true,
            Platform::MacOs,
            APP_ID,
            0,
            Vec::new(),
            RuleTarget::Exact(InputSourceId::parse("en").expect("valid id")),
        )
        .expect("valid rule")];
        let gateway = Arc::new(Mutex::new(GatewayData {
            current: "jp".to_owned(),
            ..GatewayData::default()
        }));
        let fail_save = Arc::new(AtomicBool::new(false));
        let now = Arc::new(AtomicU64::new(100));
        let versions = Arc::new(Mutex::new(Vec::new()));
        let switcher = InputSourceSwitcher::new(
            settings,
            Arc::new(Mutex::new(FakeGateway(Arc::clone(&gateway)))),
            FakeRepository {
                fail_save: Arc::clone(&fail_save),
            },
            FakeClock(Arc::clone(&now)),
            FakePublisher(Arc::clone(&versions)),
        )
        .expect("valid switcher");
        Self {
            switcher,
            gateway,
            fail_save,
            now,
            versions,
        }
    }

    fn active() -> Self {
        let mut fixture = Self::new(SwitcherSettings::defaults());
        fixture.switcher.activate(app(42), 1).expect("activate app");
        fixture.switcher.flush_publish();
        fixture
    }
}

fn app(pid: i32) -> PlatformAppId {
    PlatformAppId::new(Platform::MacOs, APP_ID, pid).expect("valid app")
}

fn snapshot(pid: i32, generation: u64) -> FocusSnapshot {
    FocusSnapshot::new(app(pid), generation, Vec::new())
}

#[test]
fn stale_generation_is_ignored() {
    let mut fixture = Fixture::new(SwitcherSettings::defaults());
    fixture.switcher.activate(app(42), 8).expect("activate");

    let decision = fixture
        .switcher
        .handle_focus(snapshot(42, 7))
        .expect("handle stale focus");

    assert_eq!(decision, SwitchDecision::Ignored);
    assert!(fixture
        .gateway
        .lock()
        .expect("gateway lock")
        .selected
        .is_empty());
}

#[test]
fn manual_change_is_respected_until_focus_changes() {
    let mut fixture = Fixture::active();

    fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("first focus");
    fixture.gateway.lock().expect("gateway lock").current = "jp".to_owned();
    fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("duplicate focus");
    assert_eq!(
        fixture.gateway.lock().expect("gateway lock").selected,
        ["en"]
    );

    fixture
        .switcher
        .handle_focus(snapshot(42, 2))
        .expect("new focus");
    assert_eq!(
        fixture.gateway.lock().expect("gateway lock").selected,
        ["en", "en"]
    );
}

#[test]
fn failed_settings_save_leaves_active_rules_unchanged() {
    let mut fixture = Fixture::active();
    let before = fixture.switcher.state().rules_version;
    fixture.fail_save.store(true, Ordering::Relaxed);
    let replacement = vec![FocusRule::new(
        "replacement",
        "Replacement",
        true,
        Platform::MacOs,
        APP_ID,
        0,
        Vec::new(),
        RuleTarget::PreferredLanguage(LanguageTag::ja()),
    )
    .expect("valid rule")];

    assert!(fixture.switcher.replace_rules(replacement).is_err());
    assert_eq!(fixture.switcher.state().rules_version, before);
    assert_eq!(fixture.switcher.rules()[0].id(), "editor");
}

#[test]
fn disabled_switcher_does_not_select() {
    let mut settings = SwitcherSettings::defaults();
    settings.enabled = false;
    let mut fixture = Fixture::new(settings);
    fixture.switcher.activate(app(42), 1).expect("activate");

    let decision = fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("handle");

    assert_eq!(decision, SwitchDecision::Disabled);
    assert_eq!(
        fixture
            .gateway
            .lock()
            .expect("gateway lock")
            .select_attempts,
        0
    );
}

#[test]
fn active_snooze_blocks_and_expired_snooze_allows_switching() {
    let mut settings = SwitcherSettings::defaults();
    settings.snoozed_until_unix_ms = Some(200);
    let mut fixture = Fixture::new(settings);
    fixture.switcher.activate(app(42), 1).expect("activate");

    assert_eq!(
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("snoozed"),
        SwitchDecision::Snoozed
    );
    fixture.now.store(201, Ordering::Relaxed);
    assert_eq!(
        fixture
            .switcher
            .handle_focus(snapshot(42, 2))
            .expect("expired"),
        SwitchDecision::Switched
    );
}

#[test]
fn indefinite_snooze_blocks_until_resume() {
    let mut fixture = Fixture::active();
    fixture
        .switcher
        .snooze_indefinitely()
        .expect("snooze indefinitely");
    // Still well below INDEFINITE_SNOOZE_UNTIL_UNIX_MS
    fixture.now.store(2_000_000_000_000, Ordering::Relaxed);

    assert_eq!(
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("still snoozed"),
        SwitchDecision::Snoozed
    );

    fixture.switcher.set_snoozed_until(None).expect("resume");
    assert_eq!(
        fixture
            .switcher
            .handle_focus(snapshot(42, 2))
            .expect("resumed"),
        SwitchDecision::Switched
    );
}

#[test]
fn same_source_still_reselects_to_refresh_context() {
    let mut fixture = Fixture::active();
    fixture.gateway.lock().expect("gateway lock").current = "en".to_owned();

    let decision = fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("handle");

    assert_eq!(decision, SwitchDecision::AlreadySelected);
    assert_eq!(
        fixture
            .gateway
            .lock()
            .expect("gateway lock")
            .select_attempts,
        1
    );
    let log = &fixture.switcher.state().logs[0];
    assert!(log.attempted);
    assert!(log.reason.contains("reselected"));
}

#[test]
fn unmatched_application_is_a_no_op() {
    let mut fixture = Fixture::active();
    let other =
        PlatformAppId::new(Platform::MacOs, "com.example.Other", 7).expect("valid other app");
    fixture
        .switcher
        .activate(other.clone(), 2)
        .expect("activate");

    let decision = fixture
        .switcher
        .handle_focus(FocusSnapshot::new(other, 2, Vec::new()))
        .expect("handle");

    assert_eq!(decision, SwitchDecision::NoRule);
    assert_eq!(
        fixture
            .gateway
            .lock()
            .expect("gateway lock")
            .select_attempts,
        0
    );
    let log = &fixture.switcher.state().logs[0];
    assert_eq!(log.decision, SwitchDecision::NoRule);
    assert!(!log.attempted);
    assert!(log.reason.contains("no matching rule"));
    assert!(log.detail.contains("elements=0"));
}

#[test]
fn switch_failure_refreshes_once_and_retries_once() {
    let mut fixture = Fixture::active();
    fixture
        .gateway
        .lock()
        .expect("gateway lock")
        .fail_first_select = true;

    let decision = fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("retry succeeds");
    let data = fixture.gateway.lock().expect("gateway lock");

    assert_eq!(decision, SwitchDecision::Switched);
    assert_eq!(data.select_attempts, 2);
    assert_eq!(data.refreshes, 1);
}

#[test]
fn switched_log_includes_post_select_verification() {
    let mut fixture = Fixture::active();
    fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("switch");
    let log = &fixture.switcher.state().logs[0];
    assert_eq!(log.decision, SwitchDecision::Switched);
    assert!(log.reason.contains("postSelect=match"));
}

#[test]
fn followup_reselect_log_only_when_debug_logging() {
    let mut fixture = Fixture::active();
    let id = InputSourceId::parse("en").expect("id");
    fixture.switcher.note_followup_reselect(40, 0, Some(&id));
    assert!(fixture.switcher.state().logs.is_empty());

    fixture.switcher.set_debug_logging(true).expect("debug on");
    fixture.switcher.note_followup_reselect(40, 0, Some(&id));
    assert!(fixture
        .switcher
        .state()
        .logs
        .iter()
        .any(|log| log.reason.contains("followup reselect")));
}

#[test]
fn switch_logs_are_forwarded_to_diagnostic_sink() {
    struct RecordingSink(Mutex<Vec<String>>);
    impl DiagnosticLogSink for RecordingSink {
        fn append_switch_log(&self, entry: &SwitchLogEntry) {
            self.0.lock().expect("sink lock").push(entry.reason.clone());
        }
    }
    let mut fixture = Fixture::active();
    let sink = Arc::new(RecordingSink(Mutex::new(Vec::new())));
    fixture
        .switcher
        .set_diagnostic_log_sink(Some(Arc::clone(&sink) as SharedDiagnosticLogSink));

    fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("switch");

    let reasons = sink.0.lock().expect("sink lock");
    assert_eq!(reasons.len(), 1);
    assert!(reasons[0].contains("selected target input source"));
}

#[test]
fn switch_log_is_capped_at_two_hundred_entries() {
    let mut fixture = Fixture::active();
    for generation in 1..=205 {
        fixture.gateway.lock().expect("gateway lock").current = "jp".to_owned();
        fixture
            .switcher
            .handle_focus(snapshot(42, generation))
            .expect("switch");
    }

    assert_eq!(fixture.switcher.state().logs.len(), 200);
}

#[test]
fn published_state_versions_are_strictly_increasing() {
    let mut fixture = Fixture::active();
    fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("first");
    fixture
        .switcher
        .handle_focus(snapshot(42, 2))
        .expect("second");
    let versions = fixture.versions.lock().expect("versions lock");

    assert!(versions.windows(2).all(|pair| pair[0] < pair[1]));
}
