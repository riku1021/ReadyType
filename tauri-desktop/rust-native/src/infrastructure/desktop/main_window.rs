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
            // main 専用（global emit だと quick-set 副窓も SPA ナビしてしまう）
            let _ = app.emit_to(MAIN_WINDOW_LABEL, NAVIGATE_EVENT, route);
        }
    }
}

/// Hide the main window and drop the Dock icon (macOS Accessory).
/// Call after any other visible windows are hidden so Accessory does not fight a foreground window.
pub fn enter_background(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.hide();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
}
