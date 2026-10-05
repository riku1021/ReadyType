use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::{
    application::switcher_controller::SwitcherController,
    infrastructure::desktop::quick_set_window,
    presentation::{
        commands::input_source_switcher::IpcError,
        input_source_dto::{QuickSetSessionDto, StateDto},
    },
};

#[tauri::command]
pub fn get_quick_set_session(
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<Option<QuickSetSessionDto>, IpcError> {
    controller
        .quick_set_session()
        .map(|session| session.as_ref().map(QuickSetSessionDto::from))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn apply_quick_set_source(
    source_id: String,
    name: String,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<StateDto, IpcError> {
    controller
        .apply_quick_set_source(&source_id, &name)
        .map(|state| StateDto::from(&state))
        .map_err(IpcError::from)
}

#[tauri::command]
pub fn hide_quick_set(app: AppHandle) -> Result<(), IpcError> {
    quick_set_window::hide(&app);
    Ok(())
}

#[tauri::command]
pub fn open_quick_set_advanced(app_id: String, app: AppHandle) -> Result<(), IpcError> {
    quick_set_window::show_main_for_app(&app, &app_id);
    Ok(())
}
