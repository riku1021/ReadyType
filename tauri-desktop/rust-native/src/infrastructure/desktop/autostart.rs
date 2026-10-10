//! Login-launch registration.
//!
//! - macOS: `SMAppService.mainApp`（OS が正本。マーカー／自動 ON なし）
//! - 他 OS: `tauri-plugin-autostart`

use tauri::AppHandle;

/// Current OS login-launch registration state.
pub fn is_login_launch_enabled(app: &AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        Ok(macos::is_enabled())
    }
    #[cfg(not(target_os = "macos"))]
    {
        use tauri_plugin_autostart::ManagerExt;
        app.autolaunch()
            .is_enabled()
            .map_err(|error| error.to_string())
    }
}

/// Enable or disable login launch. Returns the resulting OS state.
pub fn set_login_launch_enabled(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        macos::set_enabled(enabled)?;
        Ok(macos::is_enabled())
    }
    #[cfg(not(target_os = "macos"))]
    {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        if enabled {
            manager.enable().map_err(|error| error.to_string())?;
        } else {
            manager.disable().map_err(|error| error.to_string())?;
        }
        manager.is_enabled().map_err(|error| error.to_string())
    }
}

/// One-shot: drop legacy LaunchAgent (`ReadyType.plist`) and migrate ON → SMAppService.
#[cfg(target_os = "macos")]
pub fn migrate_legacy_launch_agent() {
    macos::migrate_legacy_launch_agent();
}

/// Cache `keyAELaunchedAsLogInItem` once (Cocoa `applicationDidFinishLaunching` equivalent).
/// Call at the start of Tauri `setup` before other work may consume the Apple Event.
#[cfg(target_os = "macos")]
pub fn capture_login_launch_apple_event() {
    macos::ensure_login_launch_apple_event_cached();
}

/// Silent login launch: cached Apple Event (`keyAELaunchedAsLogInItem`) or legacy `--autostart`.
pub fn is_silent_login_launch(args: impl IntoIterator<Item = String>) -> bool {
    if args.into_iter().any(|argument| argument == "--autostart") {
        return true;
    }
    #[cfg(target_os = "macos")]
    {
        macos::launched_as_login_item()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Manual cold start shows the main window; login launch stays silent.
pub fn should_show_main_window_on_launch(args: impl IntoIterator<Item = String>) -> bool {
    !is_silent_login_launch(args)
}

#[cfg(target_os = "macos")]
mod macos {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::OnceLock;

    use objc2_core_services::{
        kAEOpenApplication, kCoreEventClass, keyAELaunchedAsLogInItem, keyAEPropData,
    };
    use objc2_foundation::{NSAppleEventManager, NSError};
    use objc2_service_management::{
        kSMErrorAlreadyRegistered, kSMErrorInvalidSignature, kSMErrorJobNotFound,
        kSMErrorLaunchDeniedByUser, SMAppService, SMAppServiceStatus,
    };

    const LEGACY_LAUNCH_AGENT_LABEL: &str = "ReadyType";
    const LEGACY_LAUNCH_AGENT_FILE: &str = "ReadyType.plist";
    const ERR_NOT_APP_BUNDLE: &str = "Launch at login requires a signed .app bundle (not a bare tauri dev binary). Verify with `mise run build`. See docs/macos-code-signing.md.";
    const ERR_INVALID_SIGNATURE: &str = "Launch at login requires a valid code signature. Use `mise run dev` or `mise run build` with identity `ReadyType Code Signing` (see docs/macos-code-signing.md).";

    /// Process-lifetime cache: `currentAppleEvent` may be gone after early launch.
    static LAUNCHED_AS_LOGIN_ITEM: OnceLock<bool> = OnceLock::new();

    pub(super) fn is_enabled() -> bool {
        status() == SMAppServiceStatus::Enabled
    }

    pub(super) fn set_enabled(enabled: bool) -> Result<(), String> {
        if enabled {
            register()?;
            // Consent revoked / pending in System Settings → guide the user (Apple / Ice pattern).
            if status() == SMAppServiceStatus::RequiresApproval {
                open_login_items_settings();
            }
            Ok(())
        } else {
            unregister()
        }
    }

    fn status() -> SMAppServiceStatus {
        // SAFETY: status is a process-global read on the main-app login item.
        unsafe { SMAppService::mainAppService().status() }
    }

    fn register() -> Result<(), String> {
        if !running_inside_app_bundle() {
            return Err(ERR_NOT_APP_BUNDLE.to_owned());
        }
        // SAFETY: register operates on the main-app login item for this process.
        let service = unsafe { SMAppService::mainAppService() };
        match unsafe { service.registerAndReturnError() } {
            Ok(()) => Ok(()),
            Err(error) if is_sm_error(&error, kSMErrorAlreadyRegistered) => Ok(()),
            Err(error) if is_sm_error(&error, kSMErrorLaunchDeniedByUser) => {
                open_login_items_settings();
                Ok(())
            }
            Err(error) if is_sm_error(&error, kSMErrorInvalidSignature) => {
                Err(ERR_INVALID_SIGNATURE.to_owned())
            }
            Err(error) => Err(ns_error_message(&error)),
        }
    }

    fn running_inside_app_bundle() -> bool {
        std::env::current_exe()
            .ok()
            .and_then(|exe| app_bundle_path_from_executable(&exe))
            .is_some()
    }

    /// `Foo.app/Contents/MacOS/<bin>` → `Foo.app`. Bare `target/debug/...` → `None`.
    fn app_bundle_path_from_executable(executable: &Path) -> Option<PathBuf> {
        let macos_dir = executable.parent()?;
        if macos_dir.file_name()?.to_str()? != "MacOS" {
            return None;
        }
        let contents = macos_dir.parent()?;
        if contents.file_name()?.to_str()? != "Contents" {
            return None;
        }
        let bundle = contents.parent()?;
        if bundle.extension()?.to_str()? != "app" {
            return None;
        }
        if !bundle.join("Contents").join("Info.plist").is_file() {
            return None;
        }
        Some(bundle.to_path_buf())
    }

    fn unregister() -> Result<(), String> {
        // SAFETY: unregister operates on the main-app login item for this process.
        let service = unsafe { SMAppService::mainAppService() };
        match unsafe { service.unregisterAndReturnError() } {
            Ok(()) => Ok(()),
            Err(error) if is_sm_error(&error, kSMErrorJobNotFound) => Ok(()),
            Err(error) => Err(ns_error_message(&error)),
        }
    }

    fn open_login_items_settings() {
        // SAFETY: opens System Settings → Login Items (no object lifetime concerns).
        unsafe { SMAppService::openSystemSettingsLoginItems() };
    }

    pub(super) fn ensure_login_launch_apple_event_cached() {
        let _ = launched_as_login_item();
    }

    pub(super) fn launched_as_login_item() -> bool {
        *LAUNCHED_AS_LOGIN_ITEM.get_or_init(read_launched_as_login_item_from_apple_event)
    }

    fn read_launched_as_login_item_from_apple_event() -> bool {
        let manager = NSAppleEventManager::sharedAppleEventManager();
        let Some(event) = manager.currentAppleEvent() else {
            return false;
        };
        if event.eventClass() != kCoreEventClass || event.eventID() != kAEOpenApplication {
            return false;
        }
        let Some(prop) = event.paramDescriptorForKeyword(keyAEPropData) else {
            return false;
        };
        prop.enumCodeValue() == keyAELaunchedAsLogInItem
    }

    pub(super) fn migrate_legacy_launch_agent() {
        let Some(plist_path) = legacy_launch_agent_path() else {
            return;
        };
        if !plist_path.is_file() {
            return;
        }

        unload_legacy_launch_agent();
        if let Err(error) = fs::remove_file(&plist_path) {
            eprintln!(
                "autostart: failed to remove legacy LaunchAgent {}: {error}",
                plist_path.display()
            );
            return;
        }
        eprintln!(
            "autostart: removed legacy LaunchAgent {}",
            plist_path.display()
        );

        // Presence means the previous plugin had login launch registered → keep ON via SMAppService.
        // Do not open System Settings here (silent migrate; user can toggle later if approval is needed).
        if !is_enabled() {
            if let Err(error) = register() {
                eprintln!("autostart: legacy LaunchAgent → SMAppService migrate: {error}");
            }
        }
    }

    fn legacy_launch_agent_path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(
            PathBuf::from(home)
                .join("Library/LaunchAgents")
                .join(LEGACY_LAUNCH_AGENT_FILE),
        )
    }

    fn unload_legacy_launch_agent() {
        let Some(uid) = current_uid() else {
            return;
        };
        let domain_target = format!("gui/{uid}/{LEGACY_LAUNCH_AGENT_LABEL}");
        let _ = Command::new("launchctl")
            .args(["bootout", &domain_target])
            .status();
    }

    fn current_uid() -> Option<u32> {
        let output = Command::new("id").arg("-u").output().ok()?;
        String::from_utf8(output.stdout).ok()?.trim().parse().ok()
    }

    fn is_sm_error(error: &NSError, code: u32) -> bool {
        error.code() == isize::try_from(code).unwrap_or(isize::MAX)
    }

    fn ns_error_message(error: &NSError) -> String {
        error.localizedDescription().to_string()
    }

    #[cfg(test)]
    mod path_tests {
        use super::app_bundle_path_from_executable;
        use std::fs;
        use std::path::Path;

        #[test]
        fn bare_debug_binary_is_not_an_app_bundle() {
            assert_eq!(
                app_bundle_path_from_executable(Path::new(
                    "/var/folders/x/target/debug/tauri-desktop"
                )),
                None
            );
        }

        #[test]
        fn macos_layout_with_info_plist_is_app_bundle() {
            let root = std::env::temp_dir().join(format!(
                "readytype-autostart-bundle-test-{}",
                std::process::id()
            ));
            let app = root.join("ReadyType.app");
            let macos = app.join("Contents/MacOS");
            fs::create_dir_all(&macos).expect("temp MacOS dir");
            fs::write(app.join("Contents/Info.plist"), b"test").expect("temp Info.plist");
            let exe = macos.join("tauri-desktop");
            fs::write(&exe, b"").expect("temp exe");

            assert_eq!(app_bundle_path_from_executable(&exe), Some(app));

            let _ = fs::remove_dir_all(&root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::should_show_main_window_on_launch;

    #[test]
    fn login_launch_argument_keeps_main_window_hidden() {
        // Legacy LaunchAgent / non-mac plugin arg. SMAppService login uses Apple Events (runtime).
        assert!(should_show_main_window_on_launch(["ReadyType".to_owned()]));
        assert!(!should_show_main_window_on_launch([
            "ReadyType".to_owned(),
            "--autostart".to_owned(),
        ]));
    }
}
