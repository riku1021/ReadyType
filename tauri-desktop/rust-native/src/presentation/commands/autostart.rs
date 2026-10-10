use tauri::AppHandle;

use super::input_source_switcher::IpcError;
use crate::infrastructure::desktop::autostart;

#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> Result<bool, IpcError> {
    autostart::is_login_launch_enabled(&app).map_err(to_ipc_error)
}

#[tauri::command]
pub fn set_autostart_enabled(enabled: bool, app: AppHandle) -> Result<bool, IpcError> {
    autostart::set_login_launch_enabled(&app, enabled).map_err(to_ipc_error)
}

#[tauri::command]
pub fn should_show_main_window_on_launch() -> bool {
    autostart::should_show_main_window_on_launch(std::env::args())
}

fn to_ipc_error(message: String) -> IpcError {
    IpcError { message }
}
