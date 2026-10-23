use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

use super::{InputSourceSwitcher, SwitchDecision};
use crate::{
    application::{
        ports::{
            clock::Clock,
            diagnostic_log::{DiagnosticLogSink, QuickSetLogEntry, SharedDiagnosticLogSink},
            input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
            settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
            switcher_events::SwitcherEventPublisher,
        },
        settings::SwitcherSettings,
        switch_proto,
        switcher_state::{SwitchLogEntry, SwitcherState},
    },
    domain::input_source::{
        AxAttribute, AxElementSnapshot, FocusRule, FocusSnapshot, InputSourceId, LanguageTag,
        Platform, PlatformAppId, RuleTarget,
    },
};

const APP_ID: &str = "com.example.Editor";

/// Env ablation flags are process-global; serialize tests that mutate them.
static ABLATION_ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Default)]
struct GatewayData {
    current: String,
    selected: Vec<String>,
    select_attempts: usize,
    current_reads: usize,
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
        // Non-empty so `reconcile_language_preferences` keeps ja/en test prefs.
        Ok(vec![
            AvailableInputSource {
                id: InputSourceId::parse("en").expect("en"),
                name: "English".to_owned(),
                language_code: Some("en".to_owned()),
                available: true,
            },
            AvailableInputSource {
                id: InputSourceId::parse("jp").expect("jp"),
                name: "Japanese".to_owned(),
                language_code: Some("ja".to_owned()),
                available: true,
            },
            AvailableInputSource {
                id: InputSourceId::parse("other").expect("other"),
                name: "Other".to_owned(),
                language_code: Some("fr".to_owned()),
                available: true,
            },
        ])
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        let mut data = self.0.lock().expect("gateway lock");
        data.current_reads += 1;
        InputSourceId::parse(&data.current)
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
        Self::from_settings(settings)
    }

    /// Caller-supplied rules / preferences（上書きしない）。
    fn from_settings(settings: SwitcherSettings) -> Self {
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

fn with_clean_ablation_env<R>(f: impl FnOnce() -> R) -> R {
    let _guard = ABLATION_ENV_LOCK.lock().expect("ablation env lock");
    // SAFETY: serialized by ABLATION_ENV_LOCK.
    unsafe {
        std::env::remove_var("READYTYPE_ABLATION_NO_FOLLOWUP");
        std::env::remove_var("READYTYPE_ABLATION_NO_SAME_SOURCE_RESELECT");
        std::env::remove_var("READYTYPE_ABLATION_NO_RETRY");
    }
    f()
}

fn app(pid: i32) -> PlatformAppId {
    PlatformAppId::new(Platform::MacOs, APP_ID, pid).expect("valid app")
}

fn snapshot(pid: i32, generation: u64) -> FocusSnapshot {
    FocusSnapshot::new(app(pid), generation, Vec::new())
}

fn field_snapshot(pid: i32, generation: u64, field_id: &str) -> FocusSnapshot {
    FocusSnapshot::new(
        app(pid),
        generation,
        vec![AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextField")
            .with(AxAttribute::Identifier, field_id)],
    )
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
    with_clean_ablation_env(|| {
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
    });
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
    with_clean_ablation_env(|| {
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
    });
}

#[test]
fn ablation_no_retry_fails_without_second_select() {
    with_clean_ablation_env(|| {
        // SAFETY: serialized by ABLATION_ENV_LOCK via with_clean_ablation_env.
        unsafe {
            std::env::set_var("READYTYPE_ABLATION_NO_RETRY", "1");
        }
        let mut fixture = Fixture::active();
        fixture
            .gateway
            .lock()
            .expect("gateway lock")
            .fail_first_select = true;

        let err = fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect_err("retry disabled");
        let _ = err;
        let data = fixture.gateway.lock().expect("gateway lock");
        assert_eq!(data.select_attempts, 1);
        assert_eq!(data.refreshes, 0);
    });
}

#[test]
fn ablation_no_same_source_reselect_skips_select() {
    with_clean_ablation_env(|| {
        unsafe {
            std::env::set_var("READYTYPE_ABLATION_NO_SAME_SOURCE_RESELECT", "1");
        }
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
            0
        );
        let log = &fixture.switcher.state().logs[0];
        assert!(log.reason.contains("ablation"));
        assert_eq!(log.select_micros, 0);
    });
}

#[test]
fn lab_fields_select_preferred_languages_in_order() {
    use crate::domain::input_source::{
        readytype_lab_switch_rules, AxAttribute, AxElementSnapshot, LAB_SWITCH_FIELD_IDS,
        READYTYPE_BUNDLE_ID,
    };

    with_clean_ablation_env(|| {
        let mut settings = SwitcherSettings::defaults();
        settings.language_preferences.insert(
            LanguageTag::ja(),
            Some(InputSourceId::parse("jp").expect("jp")),
        );
        settings.language_preferences.insert(
            LanguageTag::en(),
            Some(InputSourceId::parse("en").expect("en")),
        );
        settings.rules = readytype_lab_switch_rules();
        let mut fixture = Fixture::from_settings(settings);
        let app =
            PlatformAppId::new(Platform::MacOs, READYTYPE_BUNDLE_ID, 99).expect("readytype app");
        fixture.switcher.activate(app.clone(), 1).expect("activate");
        fixture.switcher.flush_publish();

        for (index, field_id) in LAB_SWITCH_FIELD_IDS.iter().enumerate() {
            let generation = u64::try_from(index + 1).expect("gen");
            let expected = if (index + 1) % 2 == 1 { "jp" } else { "en" };
            fixture.gateway.lock().expect("gateway lock").current = "other".to_owned();
            let focused = AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXTextField")
                .with(AxAttribute::Identifier, *field_id);
            let snap = FocusSnapshot::new(app.clone(), generation, vec![focused]);
            let decision = fixture
                .switcher
                .handle_focus(snap)
                .expect("lab field switch");
            assert_eq!(
                decision,
                SwitchDecision::Switched,
                "field {field_id} decision; logs={:?}",
                fixture.switcher.state().logs.first().map(|log| &log.reason)
            );
            assert_eq!(
                fixture
                    .gateway
                    .lock()
                    .expect("gateway lock")
                    .selected
                    .last()
                    .map(String::as_str),
                Some(expected),
                "field {field_id}"
            );
        }
    });
}

#[test]
fn switch_log_records_path_timing_fields() {
    with_clean_ablation_env(|| {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("switch");
        let log = &fixture.switcher.state().logs[0];
        assert!(log.prepare_micros > 0 || log.processing_micros > 0);
        // Fake gateway is instantaneous; allow 0 on sub-micro clocks, but select must run.
        assert!(log.select_micros > 0 || log.attempted);
        assert!(log.attempted);
    });
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
fn rejected_focus_is_written_to_the_diagnostic_log() {
    let mut fixture = Fixture::active();
    let sink = recording_sink();
    fixture
        .switcher
        .set_diagnostic_log_sink(Some(Arc::clone(&sink) as SharedDiagnosticLogSink));

    fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("switch");
    fixture
        .switcher
        .handle_focus(snapshot(42, 1))
        .expect("duplicate focus");

    let reasons = sink.0.lock().expect("sink lock");
    assert!(reasons
        .iter()
        .any(|reason| reason.contains("already handled")));
}

#[test]
fn quick_set_launch_tree_is_written_to_the_diagnostic_log() {
    let mut fixture = Fixture::active();
    let sink = Arc::new(QuickSetRecordingSink(Mutex::new(Vec::new())));
    fixture
        .switcher
        .set_diagnostic_log_sink(Some(Arc::clone(&sink) as SharedDiagnosticLogSink));
    let snapshot = field_snapshot(42, 3, "composer");

    fixture
        .switcher
        .note_quick_set_launch("CommandOrControl+Shift+L", "ready", Some(&snapshot));

    let lines = sink.0.lock().expect("sink lock");
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("CommandOrControl+Shift+L"));
    assert!(lines[0].contains("ready"));
    assert!(lines[0].contains("AXTextField"));
    assert!(lines[0].contains("composer"));
}

struct QuickSetRecordingSink(Mutex<Vec<String>>);

impl DiagnosticLogSink for QuickSetRecordingSink {
    fn append_switch_log(&self, _entry: &SwitchLogEntry) {}

    fn append_quick_set(&self, entry: &QuickSetLogEntry) {
        let roles = entry
            .elements
            .iter()
            .map(|element| format!("{}:{}", element.role, element.identifier))
            .collect::<Vec<_>>()
            .join("/");
        self.0
            .lock()
            .expect("sink lock")
            .push(format!("{} {} {roles}", entry.shortcut, entry.outcome));
    }
}

#[test]
fn followup_reselect_is_written_to_the_diagnostic_log() {
    let mut fixture = Fixture::active();
    let sink = recording_sink();
    fixture
        .switcher
        .set_diagnostic_log_sink(Some(Arc::clone(&sink) as SharedDiagnosticLogSink));
    let id = InputSourceId::parse("en").expect("id");

    fixture.switcher.note_followup_reselect(40, 0, Some(&id));

    let reasons = sink.0.lock().expect("sink lock");
    assert!(reasons
        .iter()
        .any(|reason| reason.contains("followup reselect")));
}

fn recording_sink() -> Arc<RecordingSink> {
    Arc::new(RecordingSink(Mutex::new(Vec::new())))
}

struct RecordingSink(Mutex<Vec<String>>);

impl DiagnosticLogSink for RecordingSink {
    fn append_switch_log(&self, entry: &SwitchLogEntry) {
        self.0.lock().expect("sink lock").push(entry.reason.clone());
    }
}

#[test]
fn switch_logs_are_forwarded_to_diagnostic_sink() {
    let mut fixture = Fixture::active();
    let sink = recording_sink();
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

#[test]
fn publish_fingerprint_off_still_publishes_log_only_bumps() {
    switch_proto::with_clean_proto_env(|| {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("switch");
        let before = fixture.versions.lock().expect("versions").len();
        fixture.switcher.clear_logs();
        fixture.switcher.flush_publish();
        assert!(fixture.versions.lock().expect("versions").len() > before);
    });
}

#[test]
fn publish_fingerprint_on_skips_unchanged_ui_state() {
    switch_proto::with_clean_proto_env(|| {
        // SAFETY: PROTO_ENV_LOCK held by with_clean_proto_env.
        unsafe {
            std::env::set_var(switch_proto::ENV_PUBLISH_FINGERPRINT, "1");
        }
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("switch");
        let before = fixture.versions.lock().expect("versions").len();
        fixture.switcher.clear_logs();
        fixture.switcher.flush_publish();
        assert_eq!(fixture.versions.lock().expect("versions").len(), before);
    });
}

#[test]
fn current_cache_off_reads_gateway_each_switch() {
    switch_proto::with_clean_proto_env(|| {
        let mut fixture = Fixture::active();
        fixture.gateway.lock().expect("gateway").current_reads = 0;
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("switch");
        let first_reads = fixture.gateway.lock().expect("gateway").current_reads;
        assert!(first_reads >= 1);
        fixture.gateway.lock().expect("gateway").current = "jp".to_owned();
        fixture.gateway.lock().expect("gateway").current_reads = 0;
        fixture
            .switcher
            .handle_focus(snapshot(42, 2))
            .expect("switch");
        assert!(fixture.gateway.lock().expect("gateway").current_reads >= 1);
    });
}

#[test]
fn current_cache_on_reuses_warmed_value_for_current() {
    switch_proto::with_clean_proto_env(|| {
        unsafe {
            std::env::set_var(switch_proto::ENV_CURRENT_CACHE, "1");
        }
        let mut fixture = Fixture::active();
        fixture.gateway.lock().expect("gateway").current_reads = 0;
        fixture.switcher.warm_current_cache();
        assert_eq!(fixture.gateway.lock().expect("gateway").current_reads, 1);
        fixture.gateway.lock().expect("gateway").current_reads = 0;
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("switch");
        // Cache supplies current; postSelect verification still reads once.
        assert_eq!(fixture.gateway.lock().expect("gateway").current_reads, 1);
        assert!(fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache")
            .get()
            .is_some());
    });
}

#[test]
fn current_cache_invalidate_forces_fresh_current_read() {
    switch_proto::with_clean_proto_env(|| {
        unsafe {
            std::env::set_var(switch_proto::ENV_CURRENT_CACHE, "1");
        }
        let mut fixture = Fixture::active();
        fixture.switcher.warm_current_cache();
        fixture.switcher.invalidate_current_cache();
        assert!(fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache")
            .get()
            .is_none());
        fixture.gateway.lock().expect("gateway").current_reads = 0;
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("switch");
        // Miss + postSelect.
        assert_eq!(fixture.gateway.lock().expect("gateway").current_reads, 2);
    });
}

#[test]
fn manual_override_off_does_not_latch_on_external_change() {
    switch_proto::with_clean_proto_env(|| {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(field_snapshot(42, 1, "composer"))
            .expect("auto");
        fixture.gateway.lock().expect("gateway").current = "jp".to_owned();
        fixture
            .switcher
            .note_external_input_source_change(&InputSourceId::parse("jp").expect("jp"));
        fixture
            .switcher
            .handle_focus(field_snapshot(42, 2, "composer"))
            .expect("again");
        assert_eq!(
            fixture.gateway.lock().expect("gateway").selected,
            ["en", "en"]
        );
    });
}

#[test]
fn manual_override_on_latches_same_focus_key_after_external_change() {
    switch_proto::with_clean_proto_env(|| {
        unsafe {
            std::env::set_var(switch_proto::ENV_MANUAL_OVERRIDE, "1");
        }
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(field_snapshot(42, 1, "composer"))
            .expect("auto");
        assert_eq!(fixture.gateway.lock().expect("gateway").selected, ["en"]);
        fixture.gateway.lock().expect("gateway").current = "jp".to_owned();
        fixture
            .switcher
            .note_external_input_source_change(&InputSourceId::parse("jp").expect("jp"));
        let decision = fixture
            .switcher
            .handle_focus(field_snapshot(42, 2, "composer"))
            .expect("latched");
        assert_eq!(decision, SwitchDecision::Ignored);
        assert_eq!(fixture.gateway.lock().expect("gateway").selected, ["en"]);
        fixture.gateway.lock().expect("gateway").current = "jp".to_owned();
        fixture
            .switcher
            .handle_focus(field_snapshot(42, 3, "other"))
            .expect("new field");
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway")
                .selected
                .last()
                .map(String::as_str),
            Some("en")
        );
    });
}

#[test]
fn proto_cache_tis_and_manual_override_interaction() {
    switch_proto::with_clean_proto_env(|| {
        unsafe {
            std::env::set_var(switch_proto::ENV_CURRENT_CACHE, "1");
            std::env::set_var(switch_proto::ENV_TIS_NOTIFY, "1");
            std::env::set_var(switch_proto::ENV_MANUAL_OVERRIDE, "1");
        }
        let mut fixture = Fixture::active();
        fixture.switcher.warm_current_cache();
        fixture
            .switcher
            .handle_focus(field_snapshot(42, 1, "composer"))
            .expect("auto");
        // Simulate TIS notify path: invalidate + external change.
        fixture.switcher.invalidate_current_cache();
        fixture.gateway.lock().expect("gateway").current = "jp".to_owned();
        fixture
            .switcher
            .note_external_input_source_change(&InputSourceId::parse("jp").expect("jp"));
        let decision = fixture
            .switcher
            .handle_focus(field_snapshot(42, 2, "composer"))
            .expect("latched after tis");
        assert_eq!(decision, SwitchDecision::Ignored);
        assert!(fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache")
            .get()
            .is_none());
    });
}
