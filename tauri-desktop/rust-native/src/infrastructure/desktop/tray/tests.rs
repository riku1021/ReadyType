use super::{TrayMenuModel, SNOOZE_MINUTES};
use crate::{
    application::switcher_state::SwitcherState,
    domain::input_source::{Platform, PlatformAppId},
};

#[test]
fn tray_model_exposes_resume_only_during_snooze() {
    let state = SwitcherState::new(true, Some(15));
    let model = TrayMenuModel::from_state(&state, "ja", false);
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
    let model = TrayMenuModel::from_state(&state, "en", false);
    assert_eq!(model.status, "Paused");
    assert!(model
        .items
        .iter()
        .any(|item| item.id == "quit" && item.label == "Quit"));
}

#[test]
fn tray_model_uses_chinese_labels() {
    let state = SwitcherState::new(true, Some(15));
    let model = TrayMenuModel::from_state(&state, "zh", false);
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
    let model = TrayMenuModel::from_state(&state, "ja", false);
    assert!(model.status.contains("com.example.Editor"));
    assert!(model.status.contains("editor-rule"));
    assert!(model
        .items
        .iter()
        .any(|item| item.id == "toggle" && item.label.contains("OFF")));

    state.enabled = false;
    let disabled = TrayMenuModel::from_state(&state, "en", false);
    assert!(disabled.status.contains("OFF"));
    assert!(disabled
        .items
        .iter()
        .any(|item| item.id == "toggle" && item.label.contains("ON")));
}

#[test]
fn tray_model_shows_install_only_when_update_is_ready() {
    let state = SwitcherState::new(true, None);
    let idle = TrayMenuModel::from_state(&state, "ja", false);
    assert!(idle.items.iter().all(|item| item.id != "install-update"));
    let ready = TrayMenuModel::from_state(&state, "ja", true);
    assert!(ready.items.iter().any(|item| {
        item.id == "install-update" && item.label == "再起動してインストール" && item.enabled
    }));
    let quit_after_install = ready
        .items
        .iter()
        .position(|item| item.id == "install-update")
        .is_some_and(|index| {
            ready
                .items
                .get(index + 1)
                .is_some_and(|item| item.id == "quit")
        });
    assert!(quit_after_install);
}

#[test]
fn tray_has_expected_snooze_durations() {
    assert_eq!(SNOOZE_MINUTES, [5, 15, 60]);
}
