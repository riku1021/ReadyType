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

/// Lab 切替スモーク（`cargo run --bin lab-smoke --features lab-smoke`）。
/// `cargo test` では AppKit main runloop を回せないため、こちらを使う。
#[cfg(all(target_os = "macos", feature = "lab-smoke"))]
pub fn run_lab_switch_smoke() -> Result<(), String> {
    infrastructure::platform::macos::lab_switch_window::smoke::run_from_process_main()
}

use std::sync::Arc;

use application::ports::{
    diagnostic_log::{SharedDiagnosticLogSink, StartupStageLogEntry},
    input_source::InputSourceGateway,
    platform::PlatformGateway,
    settings_repository::SettingsRepository,
    switcher_events::SwitcherEventPublisher,
};
use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri::Emitter;

/// Composition helper kept so platform adapters stay wired into the crate graph.
#[allow(dead_code)]
fn current_platform_name() -> &'static str {
    infrastructure::platform::CurrentPlatform.display_name()
}

fn log_startup_stage(
    sink: Option<&SharedDiagnosticLogSink>,
    stage: &str,
    outcome: &str,
    started: std::time::Instant,
    error: Option<&str>,
) {
    let elapsed = started.elapsed();
    let entry = StartupStageLogEntry::new(
        stage,
        outcome,
        u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX),
        error,
    );
    if let Some(sink) = sink {
        sink.append_startup_stage(&entry);
    }
    match error {
        Some(error) => eprintln!(
            "startup: stage={stage} outcome={outcome} elapsed_ms={} error={error}",
            elapsed.as_millis()
        ),
        None => eprintln!(
            "startup: stage={stage} outcome={outcome} elapsed_ms={}",
            elapsed.as_millis()
        ),
    }
}

fn log_startup_error<E: std::fmt::Display>(
    sink: Option<&SharedDiagnosticLogSink>,
    stage: &str,
    started: std::time::Instant,
    error: &E,
) {
    let message = error.to_string();
    log_startup_stage(sink, stage, "error", started, Some(&message));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("startup: native_entrypoint entered");
    let _ = current_platform_name();

    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        // Quick Set NSPanel (fullscreen Space overlay). Main window stays a regular NSWindow.
        builder = builder.plugin(tauri_nspanel::init());
    }

    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        );
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            eprintln!("startup: tauri_setup entered");
            eprintln!("startup: diagnostic_log_open started");
            let diagnostic_sink = match infrastructure::persistence::FileDiagnosticLogSink::open_default() {
                Ok(sink) => {
                    eprintln!(
                        "input-source diagnostics: writing {}",
                        sink.path().display()
                    );
                    Some(Arc::new(sink) as SharedDiagnosticLogSink)
                }
                Err(error) => {
                    eprintln!("input-source diagnostics: file log disabled ({error})");
                    None
                }
            };
            let setup_started = std::time::Instant::now();
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "setup",
                "started",
                setup_started,
                None,
            );

            // AE は消費され得るので setup 先頭で一度だけキャッシュ（DidFinishLaunching 相当）。
            #[cfg(target_os = "macos")]
            infrastructure::desktop::autostart::capture_login_launch_apple_event();

            let settings_started = std::time::Instant::now();
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "settings_load",
                "started",
                settings_started,
                None,
            );
            let config_directory = app.path().app_config_dir().inspect_err(|error| {
                log_startup_error(
                    diagnostic_sink.as_ref(),
                    "settings_load",
                    settings_started,
                    error,
                );
            })?;
            std::fs::create_dir_all(&config_directory).inspect_err(|error| {
                log_startup_error(
                    diagnostic_sink.as_ref(),
                    "settings_load",
                    settings_started,
                    error,
                );
            })?;
            let repository: Arc<dyn SettingsRepository> =
                Arc::new(infrastructure::persistence::JsonSettingsRepository::new(
                    config_directory.join("input-source-switcher.json"),
                ));
            let load_outcome = repository.load().inspect_err(|error| {
                log_startup_error(
                    diagnostic_sink.as_ref(),
                    "settings_load",
                    settings_started,
                    error,
                );
            })?;
            if let Some(warning) = load_outcome.warning() {
                eprintln!("settings: {warning}");
            }
            let settings = load_outcome.settings().clone();
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "settings_load",
                "ok",
                settings_started,
                None,
            );
            let update_runtime = infrastructure::desktop::app_update::AppUpdateRuntime::load(
                app.handle().clone(),
                infrastructure::persistence::AppUpdateScheduleStore::new(
                    config_directory.join("app-update.json"),
                ),
            );
            app.manage(Arc::clone(&update_runtime));
            update_runtime.spawn();
            let quick_set_shortcut = settings.quick_set_shortcut.clone();
            let ui_locale_preference = settings.ui_locale.clone();
            let resolved_ui_locale =
                application::settings::SwitcherSettings::resolve_ui_locale(&ui_locale_preference);
            let adapters_started = std::time::Instant::now();
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "platform_adapters",
                "started",
                adapters_started,
                None,
            );
            let gateway: Box<dyn InputSourceGateway> = {
                #[cfg(target_os = "macos")]
                {
                    Box::new(
                        infrastructure::platform::input_source::MacOsInputSourceGateway::new(
                            &resolved_ui_locale,
                        )?,
                    )
                }
                #[cfg(target_os = "windows")]
                {
                    Box::new(infrastructure::platform::input_source::WindowsInputSourceGateway::new())
                }
                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    Box::new(infrastructure::platform::input_source::UnsupportedInputSourceGateway)
                }
            };
            let monitor: Box<dyn application::ports::focus_monitor::FocusMonitor> = {
                #[cfg(target_os = "macos")]
                {
                    Box::new(infrastructure::platform::macos::MacOsFocusMonitor::new())
                }
                #[cfg(target_os = "windows")]
                {
                    Box::new(infrastructure::platform::windows::WindowsFocusMonitor::default())
                }
                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    Box::new(infrastructure::platform::focus_monitor::UnsupportedFocusMonitor)
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
                    Arc::new(
                        infrastructure::platform::app_display_name_cache::CachedAppDisplayNameResolver::new(
                            infrastructure::platform::macos::MacOsAppDisplayNameResolver,
                        ),
                    )
                }
                #[cfg(target_os = "windows")]
                {
                    Arc::new(
                        infrastructure::platform::app_display_name_cache::CachedAppDisplayNameResolver::new(
                            infrastructure::platform::windows::WindowsAppDisplayNameResolver,
                        ),
                    )
                }
                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    Arc::new(
                        infrastructure::platform::app_display_name::UnsupportedAppDisplayNameResolver,
                    )
                }
            };
            let tis_change_monitor: Box<
                dyn application::ports::current_input_source_change::CurrentInputSourceChangeMonitor,
            > = {
                #[cfg(target_os = "macos")]
                {
                    Box::new(
                        infrastructure::platform::macos::MacOsCurrentInputSourceChangeMonitor::default(
                        ),
                    )
                }
                #[cfg(target_os = "windows")]
                {
                    Box::new(
                        infrastructure::platform::windows::WindowsCurrentInputSourceChangeMonitor::default(),
                    )
                }
                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                {
                    Box::new(
                        application::ports::current_input_source_change::NoopCurrentInputSourceChangeMonitor,
                    )
                }
            };
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "platform_adapters",
                "ok",
                adapters_started,
                None,
            );
            let controller_started = std::time::Instant::now();
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "controller_initialize",
                "started",
                controller_started,
                None,
            );
            let controller = application::switcher_controller::SwitcherController::new(
                application::switcher_controller::SwitcherControllerDeps {
                    settings,
                    gateway,
                    repository: Arc::clone(&repository),
                    clock: Arc::new(application::switcher_controller::SystemClock),
                    publisher,
                    monitor,
                    app_display_names: Arc::clone(&app_display_name_resolver),
                    tis_change_monitor,
                },
            )
            .inspect_err(|error| {
                log_startup_error(
                    diagnostic_sink.as_ref(),
                    "controller_initialize",
                    controller_started,
                    error,
                );
            })?;
            controller.set_diagnostic_log_sink(diagnostic_sink.clone());
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "controller_initialize",
                "ok",
                controller_started,
                None,
            );
            #[cfg(target_os = "macos")]
            {
                controller.bind_followup_reselect_log(|sink| {
                    infrastructure::platform::macos::set_followup_reselect_log_sink(Some(sink));
                });
            }
            let initial_state = controller.state()?;
            let startup_controller = Arc::clone(&controller);
            app.manage(controller);
            app.manage(app_display_name_resolver);

            let icon_cache_dir = app.path().app_cache_dir()?.join("app-icons");
            std::fs::create_dir_all(&icon_cache_dir)?;
            let app_icon_resolver: Arc<dyn application::ports::app_icon::AppIconResolver> = {
                #[cfg(target_os = "macos")]
                {
                    Arc::new(
                        infrastructure::platform::app_icon_cache::CachedAppIconResolver::with_style(
                            infrastructure::platform::macos::MacOsAppIconSource,
                            icon_cache_dir,
                            infrastructure::platform::macos::macos_icon_style_source(),
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

            #[cfg(target_os = "macos")]
            {
                let handle = app.handle().clone();
                infrastructure::platform::macos::start_icon_style_monitor(move |style_token| {
                    let _ = handle.emit(
                        presentation::commands::app_presentations::ICON_STYLE_CHANGED_EVENT,
                        presentation::commands::app_presentations::IconStyleChangedDto {
                            style_token,
                        },
                    );
                });
            }

            #[cfg(target_os = "macos")]
            {
                let handle = app.handle().clone();
                infrastructure::platform::macos::start_accessibility_permission_monitor(move || {
                    let controller = handle
                        .state::<Arc<application::switcher_controller::SwitcherController>>();
                    let Ok(permission) = controller.inner().refresh_permission() else {
                        return;
                    };
                    let _ = handle.emit(
                        presentation::commands::input_source_switcher::ACCESSIBILITY_PERMISSION_EVENT,
                        presentation::input_source_dto::PermissionDto::from(permission),
                    );
                });
            }

            #[cfg(target_os = "macos")]
            {
                let handle = app.handle().clone();
                let last_emitted_ids = std::sync::Mutex::new(String::new());
                infrastructure::platform::macos::start_enabled_input_sources_monitor(move || {
                    let controller = handle
                        .state::<Arc<application::switcher_controller::SwitcherController>>();
                    // Gateway キャッシュを更新し、表示名を永続化してから FE に再取得を促す。
                    let Ok(sources) = controller.inner().list_input_sources(true) else {
                        eprintln!("input-source: re-list after notify failed");
                        return;
                    };
                    let mut ids = sources
                        .iter()
                        .map(|source| source.id.as_str())
                        .collect::<Vec<_>>();
                    ids.sort_unstable();
                    let fingerprint = ids.join(",");
                    {
                        let Ok(mut last) = last_emitted_ids.lock() else {
                            return;
                        };
                        if *last == fingerprint {
                            return;
                        }
                        *last = fingerprint.clone();
                    }
                    let revised_at_unix_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    let _ = handle.emit(
                        presentation::commands::input_source_switcher::INPUT_SOURCES_CHANGED_EVENT,
                        presentation::commands::input_source_switcher::InputSourcesChangedDto {
                            revised_at_unix_ms,
                        },
                    );
                });
            }

            let tray_started = std::time::Instant::now();
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "tray_install",
                "started",
                tray_started,
                None,
            );
            infrastructure::desktop::tray::install(
                app.handle(),
                &initial_state,
                &resolved_ui_locale,
            )
            .inspect_err(|error| {
                log_startup_error(
                    diagnostic_sink.as_ref(),
                    "tray_install",
                    tray_started,
                    error,
                );
            })?;
            log_startup_stage(
                diagnostic_sink.as_ref(),
                "tray_install",
                "ok",
                tray_started,
                None,
            );
            let _ = infrastructure::desktop::tray::set_icon_visible(
                app.handle(),
                infrastructure::desktop::tray::icon_visible_in_settings(app.handle()),
            );

            #[cfg(target_os = "macos")]
            infrastructure::desktop::autostart::migrate_legacy_launch_agent();

            let silent_login =
                infrastructure::desktop::autostart::is_silent_login_launch(std::env::args());

            // ログイン起動は Accessory（Dock なし・窓なし）。手動起動は Regular（窓＋Dock）。
            #[cfg(target_os = "macos")]
            {
                let policy = if silent_login {
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

            // ACL / フロント準備前でも手動起動では窓を出す（visible:false のフォールバック）
            if !silent_login {
                if let Some(window) = app.get_webview_window("main") {
                    let show_started = std::time::Instant::now();
                    log_startup_stage(
                        diagnostic_sink.as_ref(),
                        "main_window_show",
                        "started",
                        show_started,
                        None,
                    );
                    match window.show() {
                        Ok(()) => log_startup_stage(
                            diagnostic_sink.as_ref(),
                            "main_window_show",
                            "ok",
                            show_started,
                            None,
                        ),
                        Err(error) => log_startup_stage(
                            diagnostic_sink.as_ref(),
                            "main_window_show",
                            "error",
                            show_started,
                            Some(&error.to_string()),
                        ),
                    }
                    let focus_started = std::time::Instant::now();
                    log_startup_stage(
                        diagnostic_sink.as_ref(),
                        "main_window_focus",
                        "started",
                        focus_started,
                        None,
                    );
                    match window.set_focus() {
                        Ok(()) => log_startup_stage(
                            diagnostic_sink.as_ref(),
                            "main_window_focus",
                            "ok",
                            focus_started,
                            None,
                        ),
                        Err(error) => log_startup_error(
                            diagnostic_sink.as_ref(),
                            "main_window_focus",
                            focus_started,
                            &error,
                        ),
                    }
                } else {
                    log_startup_stage(
                        diagnostic_sink.as_ref(),
                        "main_window_show",
                        "error",
                        std::time::Instant::now(),
                        Some("main webview window was not found"),
                    );
                }
            }

            let (startup_done_sender, startup_done_receiver) = std::sync::mpsc::channel();
            let watchdog_controller = Arc::clone(&startup_controller);
            let startup_watchdog = std::thread::Builder::new()
                .name("readytype-startup-watchdog".to_owned())
                .spawn(move || {
                    if matches!(
                        startup_done_receiver.recv_timeout(std::time::Duration::from_secs(10)),
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                    ) {
                        watchdog_controller.note_startup_stage(
                            "focus_monitor_start",
                            "slow",
                            std::time::Duration::from_secs(10),
                            None,
                        );
                    }
                });
            if let Err(error) = startup_watchdog {
                eprintln!("startup: failed to spawn focus monitor watchdog: {error}");
            }

            let worker_controller = Arc::clone(&startup_controller);
            let startup_worker = std::thread::Builder::new()
                .name("readytype-startup-monitor".to_owned())
                .spawn(move || {
                    let started = std::time::Instant::now();
                    worker_controller.note_startup_stage(
                        "focus_monitor_start",
                        "started",
                        std::time::Duration::ZERO,
                        None,
                    );
                    match worker_controller.refresh_permission() {
                        Ok(_) => worker_controller.note_startup_stage(
                            "focus_monitor_start",
                            "ok",
                            started.elapsed(),
                            None,
                        ),
                        Err(error) => worker_controller.note_startup_stage(
                            "focus_monitor_start",
                            "error",
                            started.elapsed(),
                            Some(&error.to_string()),
                        ),
                    }
                    let _ = startup_done_sender.send(());
                });
            if let Err(error) = startup_worker {
                eprintln!("startup: failed to spawn focus monitor worker: {error}");
                startup_controller.note_startup_stage(
                    "focus_monitor_start",
                    "error",
                    std::time::Duration::ZERO,
                    Some(&error.to_string()),
                );
            }

            let _ = infrastructure::desktop::quick_set_window::ensure_window(app.handle());

            if let Err(error) = infrastructure::desktop::quick_set_shortcut::register(
                app.handle(),
                &quick_set_shortcut,
            ) {
                eprintln!("quick-set shortcut: {error}");
            }

            log_startup_stage(
                diagnostic_sink.as_ref(),
                "setup",
                "ok",
                setup_started,
                None,
            );

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Focused(true) => {
                let controller = Arc::clone(
                    window
                        .state::<Arc<application::switcher_controller::SwitcherController>>()
                        .inner(),
                );
                std::mem::drop(tauri::async_runtime::spawn_blocking(move || {
                    #[cfg(target_os = "macos")]
                    if let Err(error) = controller.refresh_permission() {
                        eprintln!("focus event: permission refresh failed: {error}");
                    }
                    if let Err(error) = controller.refresh_resolved_ui_locale() {
                        eprintln!("focus event: locale refresh failed: {error}");
                    }
                }));
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                if window.label()
                    == infrastructure::desktop::quick_set_window::QUICK_SET_WINDOW_LABEL
                {
                    infrastructure::desktop::quick_set_window::hide(window.app_handle());
                } else {
                    // 他の可視窓が残ると Accessory と矛盾するので quick-set も先に隠す。
                    infrastructure::desktop::quick_set_window::hide(window.app_handle());
                    infrastructure::desktop::main_window::enter_background(window.app_handle());
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            presentation::commands::app_presentations::get_app_presentations,
            presentation::commands::app_update::ack_app_update_client,
            presentation::commands::app_update::check_app_update_now,
            presentation::commands::app_update::install_app_update,
            presentation::commands::app_update::set_app_update_auto_check,
            presentation::commands::app_update::set_app_update_auto_download,
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
            presentation::commands::input_source_switcher::set_show_menu_bar_icon,
            presentation::commands::input_source_switcher::set_ui_locale,
            presentation::commands::input_source_switcher::complete_onboarding,
            presentation::commands::input_source_switcher::ensure_lab_switch_rules,
            presentation::commands::input_source_switcher::open_lab_switch_window,
            presentation::commands::quick_set::get_quick_set_session,
            presentation::commands::quick_set::apply_quick_set_source,
            presentation::commands::quick_set::hide_quick_set,
            presentation::commands::quick_set::open_quick_set_advanced,
            presentation::commands::quick_set_shortcut::set_quick_set_shortcut,
            presentation::commands::sync_window_chrome::sync_window_chrome_theme,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, _event| {
            // macOS Dock: hide 後にアイコンをクリックすると Reopen が来る。
            // 可視ウィンドウが無いときだけ main を戻す（close-to-tray の定番）。
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen {
                has_visible_windows: false,
                ..
            } = _event
            {
                infrastructure::desktop::main_window::show_main_window(_app_handle, None);
            }
        });
}
