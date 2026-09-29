use crate::application::switcher_state::SwitcherState;
use std::sync::Arc;

use tauri::{
    menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::application::{
    settings::{SwitcherSettings, DEFAULT_RESOLVED_UI_LOCALE},
    switcher_controller::SwitcherController,
    tray_strings::tray_strings,
};

pub const SNOOZE_MINUTES: [u64; 3] = [5, 15, 60];
pub const TRAY_ID: &str = "readytype-main-tray";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMenuItemModel {
    pub id: &'static str,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMenuModel {
    pub status: String,
    pub items: Vec<TrayMenuItemModel>,
}

impl TrayMenuModel {
    pub fn from_state(state: &SwitcherState, locale: &str) -> Self {
        let strings = tray_strings(locale);
        let snoozed = state.snoozed_until_unix_ms.is_some();
        let status = if snoozed {
            strings.status_snoozed.clone()
        } else if state.enabled {
            match (&state.current_app, &state.matched_rule_id) {
                (Some(app), Some(rule)) => strings
                    .status_watching_app_rule
                    .replace("{app}", app.app_id())
                    .replace("{rule}", rule),
                (Some(app), None) => strings.status_watching_app.replace("{app}", app.app_id()),
                (None, _) => strings.status_watching.clone(),
            }
        } else {
            strings.status_off.clone()
        };
        Self {
            status,
            items: vec![
                item(
                    "toggle",
                    if state.enabled {
                        &strings.toggle_off
                    } else {
                        &strings.toggle_on
                    },
                    true,
                ),
                item("snooze-5", &strings.snooze_5m, state.enabled && !snoozed),
                item("snooze-15", &strings.snooze_15m, state.enabled && !snoozed),
                item("snooze-60", &strings.snooze_1h, state.enabled && !snoozed),
                item(
                    "snooze-indefinite",
                    &strings.snooze_indefinite,
                    state.enabled && !snoozed,
                ),
                item("resume", &strings.resume, snoozed),
                item("show", &strings.open_app, true),
                item("settings", &strings.open_settings, true),
                item("quit", &strings.quit, true),
            ],
        }
    }
}

fn item(id: &'static str, label: &str, enabled: bool) -> TrayMenuItemModel {
    TrayMenuItemModel {
        id,
        label: label.to_owned(),
        enabled,
    }
}

fn resolve_ui_locale(app: &AppHandle) -> String {
    app.try_state::<Arc<SwitcherController>>()
        .map(|controller| controller.resolved_ui_locale())
        .unwrap_or_else(|| DEFAULT_RESOLVED_UI_LOCALE.to_owned())
}

pub fn should_show_main_window_on_launch(args: impl IntoIterator<Item = String>) -> bool {
    !args.into_iter().any(|argument| argument == "--autostart")
}

pub fn install(app: &AppHandle, state: &SwitcherState, locale: &str) -> tauri::Result<()> {
    let menu = build_menu(app, state, locale)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("ReadyType")
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main_window(tray.app_handle(), None);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone()).icon_as_template(true);
    }
    builder.build(app)?;
    Ok(())
}

/// Rebuild the tray from an already-cloned snapshot and **resolved** locale.
/// Never locks the switcher mutex — locale comes from the argument or the
/// lock-free resolved-locale cache.
pub fn update(app: &AppHandle, state: &SwitcherState, locale: &str) -> tauri::Result<()> {
    let locale = if locale.is_empty() {
        resolve_ui_locale(app)
    } else {
        SwitcherSettings::resolve_ui_locale(locale)
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(build_menu(app, state, &locale)?))?;
        tray.set_tooltip(Some(TrayMenuModel::from_state(state, &locale).status))?;
    }
    Ok(())
}

fn build_menu(
    app: &AppHandle,
    state: &SwitcherState,
    locale: &str,
) -> tauri::Result<Menu<tauri::Wry>> {
    let model = TrayMenuModel::from_state(state, locale);
    let status = MenuItem::with_id(app, "status", &model.status, false, None::<&str>)?;
    let items = model
        .items
        .iter()
        .map(|item| MenuItem::with_id(app, item.id, &item.label, item.enabled, None::<&str>))
        .collect::<Result<Vec<_>, _>>()?;
    let separator_one = PredefinedMenuItem::separator(app)?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let separator_three = PredefinedMenuItem::separator(app)?;
    Menu::with_items(
        app,
        &[
            &status as &dyn IsMenuItem<_>,
            &separator_one,
            &items[0],
            &items[1],
            &items[2],
            &items[3],
            &items[4],
            &items[5],
            &separator_two,
            &items[6],
            &items[7],
            &separator_three,
            &items[8],
        ],
    )
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    if id == "quit" {
        app.exit(0);
        return;
    }
    if id == "show" {
        show_main_window(app, None);
        return;
    }
    if id == "settings" {
        show_main_window(app, Some("/settings"));
        return;
    }
    let controller = app.state::<Arc<SwitcherController>>();
    let result = match id {
        "toggle" => controller
            .state()
            .and_then(|state| controller.set_enabled(!state.enabled)),
        "snooze-5" => controller.set_snooze_minutes(5),
        "snooze-15" => controller.set_snooze_minutes(15),
        "snooze-60" => controller.set_snooze_minutes(60),
        "snooze-indefinite" => controller.set_snooze_indefinitely(),
        "resume" => controller.resume(),
        _ => return,
    };
    if let Ok(state) = result {
        let locale = resolve_ui_locale(app);
        let _ = update(app, &state, &locale);
    }
}

fn show_main_window(app: &AppHandle, route: Option<&str>) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        if let Some(route) = route {
            let _ = app.emit("readytype://navigate", route);
        }
    }
}

#[cfg(test)]
mod tests {
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
}
