use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::{
    application::{
        ports::switcher_events::SwitcherEventPublisher,
        settings::SwitcherSettings,
        switcher_controller::{ControllerError, SwitcherController},
        switcher_state::SwitcherState,
    },
    domain::input_source::FocusRule,
    infrastructure::desktop::quick_set_shortcut,
    presentation::input_source_dto::{
        InputSourceDto, PermissionDto, RuleDto, SettingsDto, StateDto, StateEventDto,
    },
};

fn settings_dto_with_registration(app: &AppHandle, settings: &SwitcherSettings) -> SettingsDto {
    let mut dto = SettingsDto::from(settings);
    dto.quick_set_shortcut_registered =
        quick_set_shortcut::is_registered(app, &settings.quick_set_shortcut);
    dto
}

pub const STATE_EVENT_NAME: &str = "input-source-switcher://state";

/// FE syncs i18n when OS preferred languages change under preference `system`.
pub const RESOLVED_UI_LOCALE_EVENT: &str = "ui-locale://resolved-changed";

/// FE syncs Accessibility UI when TCC trust may have changed (after delayed re-check).
pub const ACCESSIBILITY_PERMISSION_EVENT: &str = "accessibility://permission-changed";

/// FE refreshes OS input-source list when enabled sources may have changed (after delayed re-list).
pub const INPUT_SOURCES_CHANGED_EVENT: &str = "input-source://sources-changed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedUiLocaleEventDto {
    pub resolved_ui_locale: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSourcesChangedDto {
    pub revised_at_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub message: String,
}

impl From<ControllerError> for IpcError {
    fn from(error: ControllerError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

pub struct TauriSwitcherEventPublisher {
    app: AppHandle,
}

impl TauriSwitcherEventPublisher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl SwitcherEventPublisher for TauriSwitcherEventPublisher {
    fn publish(&self, state: &SwitcherState, locale: &str) {
        let _ = self.app.emit(STATE_EVENT_NAME, StateEventDto::from(state));
        let _ = crate::infrastructure::desktop::tray::update(&self.app, state, locale);
    }

    fn publish_resolved_ui_locale(&self, locale: &str) {
        let _ = self.app.emit(
            RESOLVED_UI_LOCALE_EVENT,
            ResolvedUiLocaleEventDto {
                resolved_ui_locale: locale.to_owned(),
            },
        );
    }
}

fn state(controller: &SwitcherController) -> Result<StateDto, IpcError> {
    controller
        .state()
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn get_input_source_switcher_state(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    state(&controller)
}

#[tauri::command]
pub fn set_input_source_switcher_enabled(
    enabled: bool,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .set_enabled(enabled)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn set_snooze(
    minutes: u64,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .set_snooze_minutes(minutes)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn snooze_input_source_switcher_indefinitely(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .set_snooze_indefinitely()
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn resume_input_source_switcher(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .resume()
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn list_input_sources(
    refresh: Option<bool>,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<Vec<InputSourceDto>, IpcError> {
    controller
        .list_input_sources(refresh.unwrap_or(false))
        .map(|sources| sources.into_iter().map(InputSourceDto::from).collect())
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn list_rules(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<Vec<RuleDto>, IpcError> {
    controller
        .rules()
        .map(|rules| rules.iter().map(RuleDto::from).collect())
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn create_rule(
    rule: RuleDto,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    let rule = FocusRule::try_from(rule).map_err(invalid_input)?;
    controller
        .create_rule(rule)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn update_rule(
    rule: RuleDto,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    let rule = FocusRule::try_from(rule).map_err(invalid_input)?;
    controller
        .update_rule(rule)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn delete_rule(
    id: String,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .delete_rule(&id)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn reorder_rules(
    ids: Vec<String>,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .reorder_rules(&ids)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn reset_default_rules(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .reset_default_rules()
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn get_input_source_settings(
    app: AppHandle,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<SettingsDto, IpcError> {
    controller
        .settings()
        .map(|settings| settings_dto_with_registration(&app, &settings))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn get_accessibility_permission(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<PermissionDto, IpcError> {
    controller
        .permission_status()
        .map(PermissionDto::from)
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn request_accessibility_permission(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<PermissionDto, IpcError> {
    controller
        .inner()
        .request_permission()
        .map(PermissionDto::from)
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn open_accessibility_settings(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<(), IpcError> {
    controller
        .open_accessibility_settings()
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn set_ui_locale(
    locale: String,
    app: AppHandle,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<SettingsDto, IpcError> {
    controller.set_ui_locale(&locale).map_err(IpcError::from)?;
    controller
        .settings()
        .map(|settings| settings_dto_with_registration(&app, &settings))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn complete_onboarding(
    app: AppHandle,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<SettingsDto, IpcError> {
    controller.complete_onboarding().map_err(IpcError::from)?;
    controller
        .settings()
        .map(|settings| settings_dto_with_registration(&app, &settings))
        .map_err(IpcError::from)
}

/// Lab 切替テスト用プリセットを既存ルールへ upsert（他ルールは維持）。
#[tauri::command]
pub fn ensure_lab_switch_rules(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .ensure_lab_switch_rules()
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

/// Lab 切替テスト用のネイティブ入力窓を開く。
///
/// 同一プロセスでは AX フォーカスが空のまま残ることがあるため、窓側で
/// AppKit / AX から Identifier を読み `report_lab_focus` へ渡す。
#[tauri::command]
pub fn open_lab_switch_window(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<(), IpcError> {
    let controller = Arc::clone(&controller);
    crate::infrastructure::desktop::lab_switch_window::show(Arc::new(move |identifier| {
        if let Err(error) = controller.report_lab_focus(identifier) {
            eprintln!("[lab] report_lab_focus failed: {error}");
        }
    }))
    .map_err(|message| IpcError { message })
}

fn invalid_input(message: String) -> IpcError {
    IpcError { message }
}
