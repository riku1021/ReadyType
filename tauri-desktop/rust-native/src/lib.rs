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

    let autostart = tauri_plugin_autostart::Builder::new().arg("--autostart");
    #[cfg(target_os = "macos")]
    let autostart = autostart.macos_launcher(tauri_plugin_autostart::MacosLauncher::LaunchAgent);

    tauri::Builder::default()
        .plugin(autostart.build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let config_directory = app.path().app_config_dir()?;
            std::fs::create_dir_all(&config_directory)?;
            let repository: Arc<dyn SettingsRepository> =
                Arc::new(infrastructure::persistence::JsonSettingsRepository::new(
                    config_directory.join("input-source-switcher.json"),
                ));
            let load_outcome = repository.load()?;
            if let Some(warning) = load_outcome.warning() {
                eprintln!("settings: {warning}");
            }
            let settings = load_outcome.settings().clone();
            let ui_locale_preference = settings.ui_locale.clone();
            let resolved_ui_locale =
                application::settings::SwitcherSettings::resolve_ui_locale(&ui_locale_preference);
            let gateway: Box<dyn InputSourceGateway> = {
                #[cfg(target_os = "macos")]
                {
                    Box::new(
                        infrastructure::platform::input_source::MacOsInputSourceGateway::new(
                            &resolved_ui_locale,
                        )?,
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
            let current_source_change_monitor: Box<
                dyn application::ports::current_input_source_change::CurrentInputSourceChangeMonitor,
            > = {
                #[cfg(target_os = "macos")]
                {
                    Box::new(
                        infrastructure::platform::macos::MacOsCurrentInputSourceChangeMonitor::new(),
                    )
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Box::new(
                        infrastructure::platform::current_input_source_change::UnsupportedCurrentInputSourceChangeMonitor,
                    )
                }
            };
            let publisher: Arc<dyn SwitcherEventPublisher> = Arc::new(
                presentation::commands::input_source_switcher::TauriSwitcherEventPublisher::new(
                    app.handle().clone(),
                ),
            );
            let app_display_name_resolver: Arc<
                dyn application::ports::app_display_name::AppDisplayNameResolver,
            > = {
                #[cfg(target_os = "macos")]
                {
                    Arc::new(infrastructure::platform::macos::MacOsAppDisplayNameResolver)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Arc::new(
                        infrastructure::platform::app_display_name::UnsupportedAppDisplayNameResolver,
                    )
                }
            };
            let controller = application::switcher_controller::SwitcherController::new(
                application::switcher_controller::SwitcherControllerDeps {
                    settings,
                    gateway,
                    repository: Arc::clone(&repository),
                    clock: Arc::new(application::switcher_controller::SystemClock),
                    publisher,
                    monitor,
                    current_source_change_monitor,
                    app_display_names: Arc::clone(&app_display_name_resolver),
                },
            )?;
            #[cfg(target_os = "macos")]
            {
                controller.bind_followup_reselect_log(|sink| {
                    infrastructure::platform::macos::set_followup_reselect_log_sink(Some(sink));
                });
            }
            match infrastructure::persistence::FileDiagnosticLogSink::open_default() {
                Ok(sink) => {
                    eprintln!(
                        "input-source diagnostics: writing {}",
                        sink.path().display()
                    );
                    controller.set_diagnostic_log_sink(Some(Arc::new(sink)));
                }
                Err(error) => {
                    eprintln!("input-source diagnostics: file log disabled ({error})");
                }
            }
            controller.refresh_permission()?;
            let initial_state = controller.state()?;
            app.manage(controller);
            app.manage(app_display_name_resolver);

            let icon_cache_dir = app.path().app_cache_dir()?.join("app-icons");
            std::fs::create_dir_all(&icon_cache_dir)?;
            let app_icon_resolver: Arc<dyn application::ports::app_icon::AppIconResolver> = {
                #[cfg(target_os = "macos")]
                {
                    Arc::new(
                        infrastructure::platform::app_icon_cache::CachedAppIconResolver::new(
                            infrastructure::platform::macos::MacOsAppIconSource,
                            icon_cache_dir,
                        ),
                    )
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Arc::new(
                        infrastructure::platform::app_icon_cache::CachedAppIconResolver::new(
                            infrastructure::platform::app_icon::UnsupportedAppIconSource,
                            icon_cache_dir,
                        ),
                    )
                }
            };
            app.manage(app_icon_resolver);

            infrastructure::desktop::tray::install(
                app.handle(),
                &initial_state,
                &resolved_ui_locale,
            )?;

            // Accessory だと Dock にも出ず、visible:false のウィンドウが
            // show() しても前面に来ない。通常起動は Regular、ログイン起動だけ Accessory。
            #[cfg(target_os = "macos")]
            {
                let is_autostart = std::env::args().any(|argument| argument == "--autostart");
                let policy = if is_autostart {
                    tauri::ActivationPolicy::Accessory
                } else {
                    tauri::ActivationPolicy::Regular
                };
                app.handle().set_activation_policy(policy)?;
            }

            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                // 起動直後はシステム外観に合わせておく。フロントの useThemeManager が上書きする。
                let is_dark = window
                    .theme()
                    .map(|theme| matches!(theme, tauri::Theme::Dark))
                    .unwrap_or(true);
                infrastructure::platform::macos::set_window_background_main(&window, is_dark);
            }

            // ACL / フロント準備前でも通常起動では窓を出す（visible:false のフォールバック）
            if infrastructure::desktop::tray::should_show_main_window_on_launch(std::env::args()) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            let _ = infrastructure::desktop::quick_set_window::ensure_window(app.handle());

            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
                let shortcut = "CommandOrControl+Alt+L"
                    .parse::<Shortcut>()
                    .map_err(|error| format!("invalid quick-set shortcut: {error}"))?;
                let handle = app.handle().clone();
                app.handle().global_shortcut().on_shortcut(
                    shortcut,
                    move |_app, _shortcut, event| {
                        if event.state != ShortcutState::Pressed {
                            return;
                        }
                        if let Err(error) =
                            infrastructure::desktop::quick_set_window::open(&handle)
                        {
                            eprintln!("quick-set open failed: {error}");
                        }
                    },
                )?;
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Focused(true) => {
                let controller =
                    window.state::<Arc<application::switcher_controller::SwitcherController>>();
                let _ = controller.inner().refresh_permission();
                let _ = controller.inner().refresh_resolved_ui_locale();
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                if window.label()
                    == infrastructure::desktop::quick_set_window::QUICK_SET_WINDOW_LABEL
                {
                    infrastructure::desktop::quick_set_window::hide(window.app_handle());
                } else {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            presentation::commands::app_presentations::get_app_presentations,
            presentation::commands::autostart::get_autostart_enabled,
            presentation::commands::autostart::set_autostart_enabled,
            presentation::commands::autostart::should_show_main_window_on_launch,
            presentation::commands::greet::greet,
            presentation::commands::quit_app::quit_app,
            presentation::commands::input_source_switcher::get_input_source_switcher_state,
            presentation::commands::input_source_switcher::set_input_source_switcher_enabled,
            presentation::commands::input_source_switcher::set_snooze,
            presentation::commands::input_source_switcher::snooze_input_source_switcher_indefinitely,
            presentation::commands::input_source_switcher::resume_input_source_switcher,
            presentation::commands::input_source_switcher::list_input_sources,
            presentation::commands::input_source_switcher::list_rules,
            presentation::commands::input_source_switcher::create_rule,
            presentation::commands::input_source_switcher::update_rule,
            presentation::commands::input_source_switcher::delete_rule,
            presentation::commands::input_source_switcher::reorder_rules,
            presentation::commands::input_source_switcher::reset_default_rules,
            presentation::commands::input_source_switcher::get_input_source_settings,
            presentation::commands::input_source_switcher::get_accessibility_permission,
            presentation::commands::input_source_switcher::request_accessibility_permission,
            presentation::commands::input_source_switcher::open_accessibility_settings,
            presentation::commands::input_source_switcher::start_debug_capture,
            presentation::commands::input_source_switcher::stop_debug_capture,
            presentation::commands::input_source_switcher::set_debug_logging,
            presentation::commands::input_source_switcher::set_ui_locale,
            presentation::commands::input_source_switcher::get_focus_snapshot,
            presentation::commands::input_source_switcher::list_switch_logs,
            presentation::commands::input_source_switcher::clear_switch_logs,
            presentation::commands::quick_set::get_quick_set_session,
            presentation::commands::quick_set::apply_quick_set_source,
            presentation::commands::quick_set::hide_quick_set,
            presentation::commands::quick_set::open_quick_set_advanced,
            presentation::commands::sync_window_chrome::sync_window_chrome_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
