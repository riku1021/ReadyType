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
mod tests;
