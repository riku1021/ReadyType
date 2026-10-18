use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};

use super::{drain_queued_focus, SwitcherController, SwitcherControllerDeps, WorkerMessage};
use crate::{
    application::{
        ports::{
            app_display_name::AppDisplayNameResolver,
            clock::Clock,
            focus_monitor::{
                AccessibilityPermission, FocusEventSink, FocusMonitor, FocusMonitorError,
            },
            input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
            settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
            switcher_events::SwitcherEventPublisher,
        },
        settings::SwitcherSettings,
        switcher_state::SwitcherState,
    },
    domain::input_source::{
        readytype_lab_switch_rules, AxAttribute, FocusRule, InputSourceId, LanguageTag, Platform,
        RuleTarget, LAB_SWITCH_FIELD_IDS,
    },
};

fn sample_rule(id: &str, priority: u32) -> FocusRule {
    FocusRule::new(
        id,
        id,
        true,
        Platform::MacOs,
        "com.example.app",
        priority,
        Vec::new(),
        RuleTarget::PreferredLanguage(LanguageTag::en()),
    )
    .expect("sample rule")
}

struct FakeGateway;

impl InputSourceGateway for FakeGateway {
    fn list(
        &mut self,
        _refresh: bool,
        _locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError> {
        Ok(Vec::new())
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        InputSourceId::parse("en").map_err(|error| GatewayError::Operation(error.to_string()))
    }

    fn select(&mut self, _id: &InputSourceId) -> Result<(), GatewayError> {
        Ok(())
    }
}

#[derive(Default)]
struct RecordingGatewayData {
    current: String,
    selected: Vec<String>,
}

struct RecordingGateway(Arc<Mutex<RecordingGatewayData>>);

impl InputSourceGateway for RecordingGateway {
    fn list(
        &mut self,
        _refresh: bool,
        _locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError> {
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
        let current = self.0.lock().expect("gateway").current.clone();
        InputSourceId::parse(&current).map_err(|error| GatewayError::Operation(error.to_string()))
    }

    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError> {
        let mut data = self.0.lock().expect("gateway");
        data.current = id.as_str().to_owned();
        data.selected.push(id.as_str().to_owned());
        Ok(())
    }
}

struct FakeRepository;

impl SettingsRepository for FakeRepository {
    fn load(&self) -> Result<LoadOutcome, SettingsError> {
        Ok(LoadOutcome::Defaults {
            settings: SwitcherSettings::defaults(),
        })
    }

    fn save(&self, _settings: &SwitcherSettings) -> Result<(), SettingsError> {
        Ok(())
    }
}

struct FakeClock(AtomicU64);

impl Clock for FakeClock {
    fn now_unix_ms(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }
}

struct NoopPublisher;

impl SwitcherEventPublisher for NoopPublisher {
    fn publish(&self, _state: &SwitcherState, _locale: &str) {}
}

#[derive(Default)]
struct RecordingPublisher {
    resolved_locales: Mutex<Vec<String>>,
}

impl SwitcherEventPublisher for RecordingPublisher {
    fn publish(&self, _state: &SwitcherState, _locale: &str) {}

    fn publish_resolved_ui_locale(&self, locale: &str) {
        self.resolved_locales
            .lock()
            .expect("publisher")
            .push(locale.to_owned());
    }
}

#[derive(Default)]
struct MonitorData {
    trusted: AtomicBool,
    starts: AtomicUsize,
    stops: AtomicUsize,
}

struct FakeMonitor(Arc<MonitorData>);

impl FocusMonitor for FakeMonitor {
    fn start(
        &mut self,
        _attributes: Vec<AxAttribute>,
        _sink: FocusEventSink,
    ) -> Result<(), FocusMonitorError> {
        self.0.starts.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FocusMonitorError> {
        self.0.stops.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    fn set_debug_capture(&mut self, _enabled: bool) -> Result<(), FocusMonitorError> {
        Ok(())
    }

    fn permission_status(&self) -> AccessibilityPermission {
        if self.0.trusted.load(Ordering::Acquire) {
            AccessibilityPermission::Granted
        } else {
            AccessibilityPermission::Denied
        }
    }

    fn request_permission(&self) -> Result<AccessibilityPermission, FocusMonitorError> {
        Ok(self.permission_status())
    }

    fn open_settings(&self) -> Result<(), FocusMonitorError> {
        Ok(())
    }
}

struct NoopDisplayNames;

impl AppDisplayNameResolver for NoopDisplayNames {
    fn resolve(&self, _bundle_id: &str) -> Option<String> {
        None
    }
}

fn controller(monitor: Arc<MonitorData>) -> Arc<SwitcherController> {
    controller_with(monitor, SwitcherSettings::defaults(), Box::new(FakeGateway))
}

fn controller_with(
    monitor: Arc<MonitorData>,
    settings: SwitcherSettings,
    gateway: Box<dyn InputSourceGateway>,
) -> Arc<SwitcherController> {
    controller_with_publisher(monitor, settings, gateway, Arc::new(NoopPublisher))
}

fn controller_with_publisher(
    monitor: Arc<MonitorData>,
    settings: SwitcherSettings,
    gateway: Box<dyn InputSourceGateway>,
    publisher: Arc<dyn SwitcherEventPublisher>,
) -> Arc<SwitcherController> {
    SwitcherController::new(SwitcherControllerDeps {
        settings,
        gateway,
        repository: Arc::new(FakeRepository),
        clock: Arc::new(FakeClock(AtomicU64::new(1_000))),
        publisher,
        monitor: Box::new(FakeMonitor(monitor)),
        app_display_names: Arc::new(NoopDisplayNames),
        tis_change_monitor: Box::new(
            crate::application::ports::current_input_source_change::NoopCurrentInputSourceChangeMonitor,
        ),
    })
    .expect("controller")
}

#[test]
fn permission_restoration_restarts_monitoring_without_process_restart() {
    let monitor = Arc::new(MonitorData::default());
    let controller = controller(Arc::clone(&monitor));
    controller.refresh_permission().expect("denied state");
    monitor.trusted.store(true, Ordering::Release);

    controller.refresh_permission().expect("restart");

    assert_eq!(monitor.starts.load(Ordering::Acquire), 1);
    assert!(controller.is_monitoring());
}

#[test]
fn permission_revocation_stops_monitoring_without_terminating_controller() {
    let monitor = Arc::new(MonitorData::default());
    monitor.trusted.store(true, Ordering::Release);
    let controller = controller(Arc::clone(&monitor));
    controller.refresh_permission().expect("start");
    monitor.trusted.store(false, Ordering::Release);

    controller.refresh_permission().expect("stop");

    assert_eq!(monitor.stops.load(Ordering::Acquire), 1);
    assert!(!controller.is_monitoring());
    assert!(controller.state().is_ok());
}

#[test]
fn rules_can_be_deleted_reordered_and_reset() {
    let controller = controller(Arc::new(MonitorData::default()));
    controller
        .create_rule(sample_rule("custom-a", 0))
        .expect("create a");
    controller
        .create_rule(sample_rule("custom-b", 1))
        .expect("create b");
    let first_id = controller.rules().expect("rules")[0].id().to_owned();
    controller.delete_rule(&first_id).expect("delete");
    let mut ids: Vec<_> = controller
        .rules()
        .expect("rules")
        .iter()
        .map(|rule| rule.id().to_owned())
        .collect();
    ids.reverse();
    controller.reorder_rules(&ids).expect("reorder");
    assert_eq!(controller.rules().expect("rules")[0].id(), ids[0]);

    controller.reset_default_rules().expect("reset");
    assert!(controller.rules().expect("rules").is_empty());
}

#[test]
fn debug_options_and_duplicate_rules_are_validated() {
    let controller = controller(Arc::new(MonitorData::default()));
    controller.set_debug_logging(true).expect("debug logging");
    controller.set_debug_capture(true).expect("capture");
    let settings = controller.settings().expect("settings");
    assert!(settings.debug_logging);
    assert!(controller.is_debug_capture_enabled());

    controller
        .create_rule(sample_rule("custom-dup", 0))
        .expect("create");
    let duplicate = controller.rules().expect("rules")[0].clone();
    assert!(controller.create_rule(duplicate).is_err());
    controller.clear_logs().expect("clear logs");
    assert!(controller.logs().expect("logs").is_empty());
}

#[test]
fn complete_onboarding_sets_flag() {
    let controller = controller(Arc::new(MonitorData::default()));
    assert!(
        !controller
            .settings()
            .expect("settings")
            .onboarding_completed
    );

    controller.complete_onboarding().expect("complete");

    assert!(
        controller
            .settings()
            .expect("settings")
            .onboarding_completed
    );
}

#[test]
fn zero_minute_snooze_expires_without_a_focus_event() {
    let controller = controller(Arc::new(MonitorData::default()));
    controller.set_snooze_minutes(0).expect("snooze");

    for _ in 0..100 {
        if controller
            .state()
            .expect("state")
            .snoozed_until_unix_ms
            .is_none()
        {
            return;
        }
        std::thread::yield_now();
    }
    panic!("snooze did not expire");
}

#[test]
fn ensure_lab_switch_rules_upserts_presets() {
    let controller = controller(Arc::new(MonitorData::default()));
    let before = controller.rules().expect("rules").len();
    controller
        .ensure_lab_switch_rules()
        .expect("ensure lab rules");
    let rules = controller.rules().expect("rules");
    assert!(rules.len() >= before);
    for field_ordinal in 1..=6 {
        let id = format!("preset-readytype-lab-switch-{field_ordinal:02}");
        assert!(rules.iter().any(|rule| rule.id() == id), "missing {id}");
    }
    // idempotent
    controller.ensure_lab_switch_rules().expect("ensure again");
    assert_eq!(controller.rules().expect("rules").len(), rules.len());
}

#[test]
fn report_lab_focus_requires_monitoring() {
    let controller = controller(Arc::new(MonitorData::default()));
    let err = controller
        .report_lab_focus(LAB_SWITCH_FIELD_IDS[0])
        .expect_err("not monitoring");
    assert!(err.to_string().contains("not running"));
}

#[test]
fn report_lab_focus_selects_preferred_source_per_field() {
    // SAFETY: clear ablation env so parallel ablation tests do not skip select.
    unsafe {
        std::env::remove_var("READYTYPE_ABLATION_NO_FOLLOWUP");
        std::env::remove_var("READYTYPE_ABLATION_NO_SAME_SOURCE_RESELECT");
        std::env::remove_var("READYTYPE_ABLATION_NO_RETRY");
    }
    let monitor = Arc::new(MonitorData::default());
    monitor.trusted.store(true, Ordering::Release);
    let gateway_data = Arc::new(Mutex::new(RecordingGatewayData {
        current: "other".to_owned(),
        selected: Vec::new(),
    }));
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
    let controller = controller_with(
        Arc::clone(&monitor),
        settings,
        Box::new(RecordingGateway(Arc::clone(&gateway_data))),
    );
    controller.refresh_permission().expect("start monitoring");
    assert!(controller.is_monitoring());

    for (index, field_id) in LAB_SWITCH_FIELD_IDS.iter().enumerate() {
        gateway_data.lock().expect("gateway").current = "other".to_owned();
        controller
            .report_lab_focus(field_id)
            .expect("report lab focus");
        let expected = if (index + 1) % 2 == 1 { "jp" } else { "en" };
        let mut saw = false;
        for _ in 0..200 {
            let selected = gateway_data.lock().expect("gateway").selected.clone();
            if selected.last().map(String::as_str) == Some(expected) {
                saw = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(saw, "field {field_id} should select {expected}");
    }
}

#[test]
fn drain_queued_focus_keeps_latest_and_defers_stop() {
    use crate::domain::input_source::{FocusSnapshot, Platform, PlatformAppId};

    let (sender, receiver) = mpsc::channel();
    let app = PlatformAppId::new(Platform::MacOs, "com.example.Editor", 1).expect("app");
    let first = FocusSnapshot::new(app.clone(), 1, Vec::new());
    let second = FocusSnapshot::new(app.clone(), 2, Vec::new());
    let third = FocusSnapshot::new(app, 3, Vec::new());
    sender.send(WorkerMessage::Focus(second)).expect("send");
    sender.send(WorkerMessage::Focus(third)).expect("send");
    sender.send(WorkerMessage::Stop).expect("send");

    let (latest, deferred) = drain_queued_focus(&receiver, first);
    assert_eq!(latest.generation(), 3);
    assert!(matches!(deferred, Some(WorkerMessage::Stop)));
}

#[test]
fn refresh_resolved_ui_locale_skips_when_preference_fixed() {
    let monitor = Arc::new(MonitorData::default());
    let mut settings = SwitcherSettings::defaults();
    settings.ui_locale = "en".to_owned();
    let publisher = Arc::new(RecordingPublisher::default());
    let controller =
        controller_with_publisher(monitor, settings, Box::new(FakeGateway), publisher.clone());
    assert!(controller
        .refresh_resolved_ui_locale()
        .expect("refresh")
        .is_none());
    assert!(publisher
        .resolved_locales
        .lock()
        .expect("publisher")
        .is_empty());
}

#[test]
fn refresh_resolved_ui_locale_noop_when_system_unchanged() {
    let monitor = Arc::new(MonitorData::default());
    let mut settings = SwitcherSettings::defaults();
    settings.ui_locale = "system".to_owned();
    let publisher = Arc::new(RecordingPublisher::default());
    let controller =
        controller_with_publisher(monitor, settings, Box::new(FakeGateway), publisher.clone());
    // Cache was seeded from the same OS resolve path at construction.
    assert!(controller
        .refresh_resolved_ui_locale()
        .expect("refresh")
        .is_none());
    assert!(publisher
        .resolved_locales
        .lock()
        .expect("publisher")
        .is_empty());
}

#[test]
fn refresh_resolved_ui_locale_publishes_when_cache_stale() {
    let monitor = Arc::new(MonitorData::default());
    let mut settings = SwitcherSettings::defaults();
    settings.ui_locale = "system".to_owned();
    let publisher = Arc::new(RecordingPublisher::default());
    let controller =
        controller_with_publisher(monitor, settings, Box::new(FakeGateway), publisher.clone());
    let current = controller.resolved_ui_locale();
    let stale = if current == "en" { "ja" } else { "en" };
    controller.ui_locale.store(stale);

    assert!(controller
        .refresh_resolved_ui_locale()
        .expect("refresh")
        .is_some());
    assert_eq!(controller.resolved_ui_locale(), current);
    assert_eq!(
        publisher
            .resolved_locales
            .lock()
            .expect("publisher")
            .as_slice(),
        &[current]
    );
}
