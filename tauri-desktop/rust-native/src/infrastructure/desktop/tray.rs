use crate::application::switcher_state::SwitcherState;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use tauri::{
    menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::application::{
    settings::{SwitcherSettings, DEFAULT_RESOLVED_UI_LOCALE},
    switcher_controller::SwitcherController,
    tray_strings::tray_strings,
};
use crate::infrastructure::desktop::main_window;

pub const SNOOZE_MINUTES: [u64; 3] = [5, 15, 60];
pub const TRAY_ID: &str = "readytype-main-tray";

#[derive(Clone)]
struct TrayHandles {
    status: MenuItem<tauri::Wry>,
    items: Vec<MenuItem<tauri::Wry>>,
}

struct TrayLive {
    handles: Option<TrayHandles>,
    /// 最後に UI へ適用済みのモデル。
    last_model: Option<(String, TrayMenuModel)>,
    /// 適用待ち（スケジュール済み）のモデル。同一内容の再スケジュールを防ぐ。
    desired: Option<(String, TrayMenuModel)>,
    generation: u64,
}

fn tray_live() -> &'static Mutex<TrayLive> {
    static LIVE: OnceLock<Mutex<TrayLive>> = OnceLock::new();
    LIVE.get_or_init(|| {
        Mutex::new(TrayLive {
            handles: None,
            last_model: None,
            desired: None,
            generation: 0,
        })
    })
}

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

fn should_skip_tray_update(
    last: Option<&(String, TrayMenuModel)>,
    desired: Option<&(String, TrayMenuModel)>,
    next: &(String, TrayMenuModel),
) -> bool {
    last == Some(next) || desired == Some(next)
}

pub fn install(app: &AppHandle, state: &SwitcherState, locale: &str) -> tauri::Result<()> {
    let locale = SwitcherSettings::resolve_ui_locale(locale);
    let model = TrayMenuModel::from_state(state, &locale);
    let (handles, menu) = build_handles_and_menu(app, &model)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip(&model.status)
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
                main_window::show_main_window(tray.app_handle(), None);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone()).icon_as_template(true);
    }
    builder.build(app)?;
    if let Ok(mut live) = tray_live().lock() {
        live.handles = Some(handles);
        live.last_model = Some((locale, model));
        live.desired = None;
        live.generation = 0;
    }
    Ok(())
}

/// Tray を最新状態へ合わせる。ワーカーは待たない（別スレッドで差分適用）。
///
/// - A: MenuItem を保持し `set_text` / `set_enabled` のみ（フル `set_menu` しない）
/// - B: 適用はバックグラウンド＋ latest-wins（古い generation は破棄）
///
/// `MenuItem::set_*` 自体が main へ marshal するため、`run_on_main_thread` 内で
/// `set_*` するとデッドロックする。そのため専用スレッドから呼ぶ。
pub fn update(app: &AppHandle, state: &SwitcherState, locale: &str) -> tauri::Result<()> {
    let locale = if locale.is_empty() {
        resolve_ui_locale(app)
    } else {
        SwitcherSettings::resolve_ui_locale(locale)
    };
    let model = TrayMenuModel::from_state(state, &locale);
    let next = (locale, model);

    let (generation, handles, previous) = {
        let Ok(mut live) = tray_live().lock() else {
            return Ok(());
        };
        if should_skip_tray_update(live.last_model.as_ref(), live.desired.as_ref(), &next) {
            return Ok(());
        }
        let Some(handles) = live.handles.clone() else {
            return Ok(());
        };
        live.generation = live.generation.saturating_add(1);
        let generation = live.generation;
        let previous = live.last_model.as_ref().map(|(_, model)| model.clone());
        live.desired = Some(next.clone());
        (generation, handles, previous)
    };

    let app = app.clone();
    let _ = thread::Builder::new()
        .name("readytype-tray".to_owned())
        .spawn(move || {
            if !is_latest_generation(generation) {
                return;
            }
            apply_model_diff(&app, &handles, previous.as_ref(), &next.1);
            if let Ok(mut live) = tray_live().lock() {
                if live.generation != generation {
                    return;
                }
                live.last_model = Some(next);
                if live.desired.as_ref() == live.last_model.as_ref() {
                    live.desired = None;
                }
            }
        });
    Ok(())
}

fn is_latest_generation(generation: u64) -> bool {
    tray_live()
        .lock()
        .ok()
        .is_some_and(|live| live.generation == generation)
}

fn apply_model_diff(
    app: &AppHandle,
    handles: &TrayHandles,
    previous: Option<&TrayMenuModel>,
    model: &TrayMenuModel,
) {
    let status_changed = previous.is_none_or(|prev| prev.status != model.status);
    if status_changed {
        let _ = handles.status.set_text(&model.status);
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_tooltip(Some(model.status.clone()));
        }
    }
    for (index, item_model) in model.items.iter().enumerate() {
        let Some(handle) = handles.items.get(index) else {
            continue;
        };
        let prev = previous.and_then(|prev| prev.items.get(index));
        if prev.is_none_or(|prev| prev.label != item_model.label) {
            let _ = handle.set_text(&item_model.label);
        }
        if prev.is_none_or(|prev| prev.enabled != item_model.enabled) {
            let _ = handle.set_enabled(item_model.enabled);
        }
    }
}

fn build_handles_and_menu(
    app: &AppHandle,
    model: &TrayMenuModel,
) -> tauri::Result<(TrayHandles, Menu<tauri::Wry>)> {
    let status = MenuItem::with_id(app, "status", &model.status, false, None::<&str>)?;
    let items = model
        .items
        .iter()
        .map(|item| MenuItem::with_id(app, item.id, &item.label, item.enabled, None::<&str>))
        .collect::<Result<Vec<_>, _>>()?;
    let separator_one = PredefinedMenuItem::separator(app)?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let separator_three = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
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
    )?;
    Ok((TrayHandles { status, items }, menu))
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    if id == "quit" {
        app.exit(0);
        return;
    }
    if id == "show" {
        main_window::show_main_window(app, None);
        return;
    }
    if id == "settings" {
        main_window::show_main_window(app, Some("/settings"));
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

#[cfg(test)]
mod tests {
    use super::{
        should_show_main_window_on_launch, should_skip_tray_update, TrayMenuModel, SNOOZE_MINUTES,
    };
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

    #[test]
    fn tray_update_skips_identical_last_or_desired() {
        let mut state = SwitcherState::new(true, None);
        state.current_app =
            Some(PlatformAppId::new(Platform::MacOs, "com.example.Editor", 42).expect("app"));
        state.matched_rule_id = Some("editor".to_owned());
        let model = TrayMenuModel::from_state(&state, "ja");
        let pair = ("ja".to_owned(), model);
        assert!(should_skip_tray_update(Some(&pair), None, &pair));
        assert!(should_skip_tray_update(None, Some(&pair), &pair));
        assert!(!should_skip_tray_update(None, None, &pair));
        let other = ("en".to_owned(), TrayMenuModel::from_state(&state, "en"));
        assert!(!should_skip_tray_update(Some(&pair), None, &other));
    }
}
