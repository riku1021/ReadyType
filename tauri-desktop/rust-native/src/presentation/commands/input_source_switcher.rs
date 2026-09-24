use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::{
    application::{
        ports::switcher_events::SwitcherEventPublisher,
        switcher_controller::{ControllerError, SwitcherController},
        switcher_state::SwitcherState,
    },
    domain::input_source::{FocusRule, InputSourceId},
    presentation::input_source_dto::{
        FocusSnapshotDto, InputSourceDto, PermissionDto, RuleDto, SettingsDto, StateDto,
        StateEventDto, SwitchLogDto,
    },
};

pub const STATE_EVENT_NAME: &str = "input-source-switcher://state";

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
    fn publish(&self, state: &SwitcherState) {
        let _ = self.app.emit(STATE_EVENT_NAME, StateEventDto::from(state));
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
pub fn update_input_source_preferences(
    preferred_english: String,
    preferred_japanese: String,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    let english = InputSourceId::parse(&preferred_english)
        .map_err(|error| invalid_input(error.to_string()))?;
    let japanese = InputSourceId::parse(&preferred_japanese)
        .map_err(|error| invalid_input(error.to_string()))?;
    controller
        .update_input_source_preferences(english, japanese)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn get_input_source_settings(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<SettingsDto, IpcError> {
    controller
        .settings()
        .map(|settings| SettingsDto::from(&settings))
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
pub fn start_debug_capture(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .set_debug_capture(true)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn stop_debug_capture(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .set_debug_capture(false)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn set_debug_logging(
    enabled: bool,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .set_debug_logging(enabled)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn get_focus_snapshot(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<Option<FocusSnapshotDto>, IpcError> {
    controller
        .focus_snapshot()
        .map(|snapshot| snapshot.as_ref().map(FocusSnapshotDto::from))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn list_switch_logs(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<Vec<SwitchLogDto>, IpcError> {
    controller
        .logs()
        .map(|logs| logs.into_iter().map(SwitchLogDto::from).collect())
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn clear_switch_logs(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .clear_logs()
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

fn invalid_input(message: String) -> IpcError {
    IpcError { message }
}
