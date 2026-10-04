//! macOS application display names via `NSWorkspace` + `NSFileManager.displayNameAtPath:`.

use objc2_foundation::{NSFileManager, NSString};

use crate::application::ports::app_display_name::AppDisplayNameResolver;
use crate::infrastructure::platform::macos::application_path;

/// Resolves Finder-style display names for bundle identifiers.
pub struct MacOsAppDisplayNameResolver;

impl AppDisplayNameResolver for MacOsAppDisplayNameResolver {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        let path = application_path(bundle_id)?;
        let manager = NSFileManager::defaultManager();
        let path_ns = NSString::from_str(&path);
        let name = manager.displayNameAtPath(&path_ns).to_string();
        let display = strip_app_extension(name.trim());
        if display.is_empty() {
            None
        } else {
            Some(display)
        }
    }
}

/// Finder 一覧向け。`displayNameAtPath` が返す `Foo.app` から拡張子を除く。
fn strip_app_extension(name: &str) -> String {
    const SUFFIX: &str = ".app";
    if name.len() > SUFFIX.len() && name.to_ascii_lowercase().ends_with(SUFFIX) {
        name[..name.len() - SUFFIX.len()].to_owned()
    } else {
        name.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{strip_app_extension, MacOsAppDisplayNameResolver};
    use crate::application::ports::app_display_name::AppDisplayNameResolver;

    #[test]
    fn unknown_bundle_id_is_none() {
        assert!(MacOsAppDisplayNameResolver
            .resolve("com.readytype.missing.app")
            .is_none());
    }

    #[test]
    fn strips_app_extension_from_display_name() {
        assert_eq!(strip_app_extension("Google Chrome.app"), "Google Chrome");
        assert_eq!(strip_app_extension("Terminal.APP"), "Terminal");
        assert_eq!(strip_app_extension("Cursor"), "Cursor");
        assert_eq!(strip_app_extension(".app"), ".app");
    }

    #[test]
    fn terminal_bundle_resolves_display_name() {
        let name = match MacOsAppDisplayNameResolver.resolve("com.apple.Terminal") {
            Some(value) => value,
            None => {
                eprintln!("skip: NSWorkspace could not resolve com.apple.Terminal");
                return;
            }
        };
        assert!(!name.is_empty());
        assert!(
            !name.to_ascii_lowercase().ends_with(".app"),
            "display name should not include .app: {name}"
        );
        // Localized; English is "Terminal".
        assert!(
            name.contains("Terminal") || name.contains("ターミナル"),
            "unexpected display name: {name}"
        );
    }
}
