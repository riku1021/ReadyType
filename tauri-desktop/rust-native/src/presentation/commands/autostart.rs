use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use super::input_source_switcher::IpcError;
use crate::infrastructure::desktop::tray;

#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> Result<bool, IpcError> {
    app.autolaunch().is_enabled().map_err(plugin_error)
}

#[tauri::command]
pub fn set_autostart_enabled(enabled: bool, app: AppHandle) -> Result<bool, IpcError> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(plugin_error)?;
    } else {
        manager.disable().map_err(plugin_error)?;
    }
    manager.is_enabled().map_err(plugin_error)
}

#[tauri::command]
pub fn should_show_main_window_on_launch() -> bool {
    tray::should_show_main_window_on_launch(std::env::args())
}

fn plugin_error(error: tauri_plugin_autostart::Error) -> IpcError {
    IpcError {
        message: error.to_string(),
    }
}
