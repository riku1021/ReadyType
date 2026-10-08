use super::{
    AttributePlan, MonitorState, MonitoringMode, ObserverRegistration, FALLBACK_INTERVAL_MS,
    MAX_PARENT_DEPTH,
};
use crate::domain::input_source::{AxAttribute, AxElementSnapshot, Platform, PlatformAppId};

fn app(id: &str, pid: i32) -> PlatformAppId {
    PlatformAppId::new(Platform::MacOs, id, pid).expect("valid app")
}

#[test]
fn observer_failure_enables_fallback_only_for_frontmost_pid() {
    let mut state = MonitorState::default();
    state.activate(app("com.example", 41), ObserverRegistration::Unsupported);
    assert_eq!(
        state.mode(),
        MonitoringMode::FallbackPolling {
            interval_ms: FALLBACK_INTERVAL_MS
        }
    );

    state.activate(app("com.other", 42), ObserverRegistration::Registered);

    assert_eq!(state.mode(), MonitoringMode::AxNotification);
    assert!(!state.accepts_pid(41));
    assert!(state.accepts_pid(42));
}

#[test]
fn permission_revocation_stops_observer_and_restoration_restarts() {
    let mut state = MonitorState::default();
    state.activate(app("com.example", 41), ObserverRegistration::Registered);

    state.update_permission(false);
    assert_eq!(state.mode(), MonitoringMode::StoppedNoPermission);
    assert!(!state.needs_restart());

    state.update_permission(true);
    assert_eq!(state.mode(), MonitoringMode::Stopped);
    assert!(state.needs_restart());
}

#[test]
fn activation_increments_generation_and_duplicate_signals_are_coalesced() {
    let mut state = MonitorState::default();
    state.activate(app("com.example", 41), ObserverRegistration::Registered);
    let generation = state.generation();
    assert!(state.enqueue(41));
    assert!(!state.enqueue(41));
    assert_eq!(state.take_pending(), Some(41));

    state.activate(app("com.other", 42), ObserverRegistration::Registered);
    assert!(state.generation() > generation);
    assert!(!state.enqueue(41));
}

#[test]
fn focus_fingerprint_change_advances_generation_once_per_identity() {
    let mut state = MonitorState::default();
    state.activate(app("com.example", 41), ObserverRegistration::Registered);
    let after_activate = state.generation();

    let first = state
        .commit_focus_change("textarea`aichat``")
        .expect("first focus");
    assert!(first > after_activate);
    assert_eq!(state.commit_focus_change("textarea`aichat``"), None);

    let second = state
        .commit_focus_change("textarea`editor``")
        .expect("second focus");
    assert!(second > first);
}

#[test]
fn focus_fingerprint_ignores_description_volatility() {
    let left = [AxElementSnapshot::new()
        .with(AxAttribute::Role, "AXTextArea")
        .with(AxAttribute::DomClassList, "aislash-editor-input")
        .with(AxAttribute::Description, "message A")];
    let right = [AxElementSnapshot::new()
        .with(AxAttribute::Role, "AXTextArea")
        .with(AxAttribute::DomClassList, "aislash-editor-input")
        .with(AxAttribute::Description, "message B")];
    assert_eq!(
        super::focus_fingerprint(&left),
        super::focus_fingerprint(&right)
    );
}

#[test]
fn attribute_plans_are_minimal_normally_and_complete_for_debugging() {
    let normal = AttributePlan::normal([
        AxAttribute::Role,
        AxAttribute::DomIdentifier,
        AxAttribute::Role,
    ]);
    assert_eq!(
        normal.attributes(),
        &[AxAttribute::Role, AxAttribute::DomIdentifier]
    );

    let debug = AttributePlan::debug();
    assert_eq!(debug.attributes().len(), 10);
    assert!(debug.attributes().contains(&AxAttribute::Help));
    assert!(debug.attributes().contains(&AxAttribute::Placeholder));
    assert!(debug.attributes().contains(&AxAttribute::DomClassList));
}

#[test]
fn parent_walk_is_bounded() {
    assert_eq!(MAX_PARENT_DEPTH, 14);
}
