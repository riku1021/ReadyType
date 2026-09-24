//! Composition root for the ReadyType Tauri desktop native crate.
//!
//! Layering (Onion / Clean Architecture):
//! - `domain` — entities and domain errors (innermost)
//! - `application` — use cases and outbound ports
//! - `presentation` — Tauri commands (inbound adapters)
//! - `infrastructure` — OS / persistence implementations (outermost)
//!
//! Dependency rule: outer layers may depend on inner layers; never the reverse.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use application::ports::platform::PlatformGateway;
use tauri::Manager;

/// Composition helper kept so platform adapters stay wired into the crate graph.
#[allow(dead_code)]
fn current_platform_name() -> &'static str {
    infrastructure::platform::CurrentPlatform.display_name()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = current_platform_name();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                // 起動直後はシステム外観に合わせておく。フロントの useThemeManager が上書きする。
                let is_dark = window
                    .theme()
                    .map(|theme| matches!(theme, tauri::Theme::Dark))
                    .unwrap_or(true);
                infrastructure::platform::macos::set_window_background_main(&window, is_dark);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            presentation::commands::greet::greet,
            presentation::commands::sync_window_chrome::sync_window_chrome_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
