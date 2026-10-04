//! Resolve an installed application path from a bundle identifier via `NSWorkspace`.

use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSString;

/// Returns the filesystem path for `bundle_id`, or `None` when Launch Services cannot locate it.
pub fn application_path(bundle_id: &str) -> Option<String> {
    if bundle_id.is_empty() {
        return None;
    }
    let workspace = NSWorkspace::sharedWorkspace();
    let bundle = NSString::from_str(bundle_id);
    let url = workspace.URLForApplicationWithBundleIdentifier(&bundle)?;
    Some(url.path()?.to_string())
}
