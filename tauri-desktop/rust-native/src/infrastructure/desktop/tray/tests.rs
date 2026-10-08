use super::{should_show_main_window_on_launch, TrayMenuModel, SNOOZE_MINUTES};
use crate::{
    application::switcher_state::SwitcherState,
    domain::input_source::{Platform, PlatformAppId},
};

#[test]
fn tray_model_exposes_resume_only_during_snooze() {
    let state = SwitcherState::new(true, Some(15));
    let model = TrayMenuModel::from_state(&state, "ja");
    assert!(model
        .items
        .iter()
        .any(|item| item.id == "resume" && item.enabled));
    assert!(model.status.contains("一時停止"));
    assert!(model
        .items
        .iter()
        .filter(|item| item.id.starts_with("snooze-"))
        .all(|item| !item.enabled));
}

#[test]
fn tray_model_uses_english_labels() {
    let state = SwitcherState::new(true, Some(15));
    let model = TrayMenuModel::from_state(&state, "en");
    assert_eq!(model.status, "Paused");
    assert!(model
        .items
        .iter()
        .any(|item| item.id == "quit" && item.label == "Quit"));
}

#[test]
fn tray_model_uses_chinese_labels() {
    let state = SwitcherState::new(true, Some(15));
    let model = TrayMenuModel::from_state(&state, "zh");
    assert_eq!(model.status, "已暂停");
    assert!(model
        .items
        .iter()
        .any(|item| item.id == "quit" && item.label == "退出"));
}

#[test]
fn tray_model_reflects_enabled_state_and_current_target() {
    let mut state = SwitcherState::new(true, None);
    state.current_app =
        Some(PlatformAppId::new(Platform::MacOs, "com.example.Editor", 42).expect("app"));
    state.matched_rule_id = Some("editor-rule".to_owned());
    let model = TrayMenuModel::from_state(&state, "ja");
    assert!(model.status.contains("com.example.Editor"));
    assert!(model.status.contains("editor-rule"));
    assert!(model
        .items
        .iter()
        .any(|item| item.id == "toggle" && item.label.contains("OFF")));

    state.enabled = false;
    let disabled = TrayMenuModel::from_state(&state, "en");
    assert!(disabled.status.contains("OFF"));
    assert!(disabled
        .items
        .iter()
        .any(|item| item.id == "toggle" && item.label.contains("ON")));
}

#[test]
fn tray_has_expected_snooze_durations() {
    assert_eq!(SNOOZE_MINUTES, [5, 15, 60]);
}

#[test]
fn login_launch_argument_keeps_main_window_hidden() {
    assert!(should_show_main_window_on_launch(["ReadyType".to_owned()]));
    assert!(!should_show_main_window_on_launch([
        "ReadyType".to_owned(),
        "--autostart".to_owned(),
    ]));
}
