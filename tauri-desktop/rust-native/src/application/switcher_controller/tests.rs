use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc,
};

use super::{SwitcherController, SwitcherControllerDeps};
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
    domain::input_source::{AxAttribute, InputSourceId},
};

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
    SwitcherController::new(SwitcherControllerDeps {
        settings: SwitcherSettings::defaults(),
        gateway: Box::new(FakeGateway),
        repository: Arc::new(FakeRepository),
        clock: Arc::new(FakeClock(AtomicU64::new(1_000))),
        publisher: Arc::new(NoopPublisher),
        monitor: Box::new(FakeMonitor(monitor)),
        app_display_names: Arc::new(NoopDisplayNames),
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
    assert!(controller
        .rules()
        .expect("rules")
        .iter()
        .any(|rule| rule.id() == first_id));
}

#[test]
fn debug_options_and_duplicate_rules_are_validated() {
    let controller = controller(Arc::new(MonitorData::default()));
    controller.set_debug_logging(true).expect("debug logging");
    controller.set_debug_capture(true).expect("capture");
    let settings = controller.settings().expect("settings");
    assert!(settings.debug_logging);
    assert!(controller.is_debug_capture_enabled());

    let duplicate = controller.rules().expect("rules")[0].clone();
    assert!(controller.create_rule(duplicate).is_err());
    controller.clear_logs().expect("clear logs");
    assert!(controller.logs().expect("logs").is_empty());
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
