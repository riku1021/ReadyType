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

use std::sync::Arc;

use application::ports::{
    input_source::InputSourceGateway, platform::PlatformGateway,
    settings_repository::SettingsRepository, switcher_events::SwitcherEventPublisher,
};
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
            let config_directory = app.path().app_config_dir()?;
            std::fs::create_dir_all(&config_directory)?;
            let repository: Arc<dyn SettingsRepository> =
                Arc::new(infrastructure::persistence::JsonSettingsRepository::new(
                    config_directory.join("input-source-switcher.json"),
                ));
            let settings = repository.load()?.settings().clone();
            let gateway: Box<dyn InputSourceGateway> = {
                #[cfg(target_os = "macos")]
                {
                    Box::new(
                        infrastructure::platform::input_source::MacOsInputSourceGateway::new()?,
                    )
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Box::new(infrastructure::platform::input_source::UnsupportedInputSourceGateway)
                }
            };
            let monitor: Box<dyn application::ports::focus_monitor::FocusMonitor> = {
                #[cfg(target_os = "macos")]
                {
                    Box::new(infrastructure::platform::macos::MacOsFocusMonitor::new())
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Box::new(infrastructure::platform::focus_monitor::UnsupportedFocusMonitor)
                }
            };
            let publisher: Arc<dyn SwitcherEventPublisher> = Arc::new(
                presentation::commands::input_source_switcher::TauriSwitcherEventPublisher::new(
                    app.handle().clone(),
                ),
            );
            let controller = application::switcher_controller::SwitcherController::new(
                settings,
                gateway,
                Arc::clone(&repository),
                Arc::new(application::switcher_controller::SystemClock),
                publisher,
                monitor,
            )?;
            controller.refresh_permission()?;
            app.manage(controller);

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
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Focused(true)) {
                let controller =
                    window.state::<Arc<application::switcher_controller::SwitcherController>>();
                let _ = controller.inner().refresh_permission();
            }
        })
        .invoke_handler(tauri::generate_handler![
            presentation::commands::greet::greet,
            presentation::commands::input_source_switcher::get_input_source_switcher_state,
            presentation::commands::input_source_switcher::set_input_source_switcher_enabled,
            presentation::commands::input_source_switcher::set_snooze,
            presentation::commands::input_source_switcher::resume_input_source_switcher,
            presentation::commands::input_source_switcher::list_input_sources,
            presentation::commands::input_source_switcher::list_rules,
            presentation::commands::input_source_switcher::create_rule,
            presentation::commands::input_source_switcher::update_rule,
            presentation::commands::input_source_switcher::delete_rule,
            presentation::commands::input_source_switcher::reorder_rules,
            presentation::commands::input_source_switcher::reset_default_rules,
            presentation::commands::input_source_switcher::update_input_source_preferences,
            presentation::commands::input_source_switcher::get_input_source_settings,
            presentation::commands::input_source_switcher::get_accessibility_permission,
            presentation::commands::input_source_switcher::request_accessibility_permission,
            presentation::commands::input_source_switcher::open_accessibility_settings,
            presentation::commands::input_source_switcher::start_debug_capture,
            presentation::commands::input_source_switcher::stop_debug_capture,
            presentation::commands::input_source_switcher::set_debug_logging,
            presentation::commands::input_source_switcher::get_focus_snapshot,
            presentation::commands::input_source_switcher::list_switch_logs,
            presentation::commands::input_source_switcher::clear_switch_logs,
            presentation::commands::sync_window_chrome::sync_window_chrome_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
