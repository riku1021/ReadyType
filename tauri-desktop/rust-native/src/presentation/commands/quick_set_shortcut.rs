//! Quick Set グローバルショートカットの更新。

use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::{
    application::switcher_controller::SwitcherController,
    infrastructure::desktop::quick_set_shortcut,
    presentation::commands::input_source_switcher::IpcError,
};

#[tauri::command]
pub fn set_quick_set_shortcut(
    shortcut: String,
    app: AppHandle,
    controller: State<'_, Arc<SwitcherController>>,
) -> Result<String, IpcError> {
    let previous = controller
        .settings()
        .map(|settings| settings.quick_set_shortcut.clone())
        .map_err(IpcError::from)?;

    let normalized = crate::application::quick_set_shortcut::normalize_quick_set_shortcut(
        &shortcut,
    )
    .map_err(|error| IpcError {
        message: error.to_string(),
    })?;

    quick_set_shortcut::replace(&app, &previous, &normalized)
        .map_err(|message| IpcError { message })?;

    if let Err(error) = controller.set_quick_set_shortcut(&normalized) {
        let _ = quick_set_shortcut::replace(&app, &normalized, &previous);
        return Err(IpcError::from(error));
    }

    Ok(normalized)
}
