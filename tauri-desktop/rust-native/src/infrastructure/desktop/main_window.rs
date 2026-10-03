//! Main window show / navigate helpers shared by tray and Quick Set.

use tauri::{AppHandle, Emitter, Manager};

pub const NAVIGATE_EVENT: &str = "readytype://navigate";
pub const MAIN_WINDOW_LABEL: &str = "main";

/// Show the main window (and optionally navigate the SPA).
pub fn show_main_window(app: &AppHandle, route: Option<&str>) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    }
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        if let Some(route) = route {
            let _ = app.emit(NAVIGATE_EVENT, route);
        }
    }
}
