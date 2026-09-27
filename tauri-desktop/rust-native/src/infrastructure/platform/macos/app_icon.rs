//! macOS application icon source backed by `NSWorkspace`.

use objc2::AnyThread;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
use objc2_foundation::{NSDictionary, NSString};

use crate::infrastructure::platform::app_icon_cache::{
    file_mtime_secs, AppIconSource, FetchedIcon,
};

/// Resolves the OS icon for a bundle id into PNG bytes + mtime token.
pub struct MacOsAppIconSource;

impl AppIconSource for MacOsAppIconSource {
    fn probe_mtime(&self, bundle_id: &str) -> Option<u64> {
        let path = application_path(bundle_id)?;
        file_mtime_secs(std::path::Path::new(path.as_str()))
    }

    fn fetch(&self, bundle_id: &str) -> Option<FetchedIcon> {
        let path = application_path(bundle_id)?;
        let mtime_secs = file_mtime_secs(std::path::Path::new(path.as_str()))?;
        let png = icon_png_bytes_for_path(&path)?;
        Some(FetchedIcon { png, mtime_secs })
    }
}

fn application_path(bundle_id: &str) -> Option<String> {
    let workspace = NSWorkspace::sharedWorkspace();
    let bundle = NSString::from_str(bundle_id);
    let url = workspace.URLForApplicationWithBundleIdentifier(&bundle)?;
    Some(url.path()?.to_string())
}

fn icon_png_bytes_for_path(path: &str) -> Option<Vec<u8>> {
    let workspace = NSWorkspace::sharedWorkspace();
    let path_ns = NSString::from_str(path);
    let image = workspace.iconForFile(&path_ns);
    let tiff = image.TIFFRepresentation()?;

    let rep = NSBitmapImageRep::initWithData(NSBitmapImageRep::alloc(), &tiff)?;
    let properties = NSDictionary::new();
    let png =
        unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties) }?;

    Some(unsafe { png.as_bytes_unchecked() }.to_vec())
}

#[cfg(test)]
mod tests {
    use super::MacOsAppIconSource;
    use crate::infrastructure::platform::app_icon_cache::AppIconSource;

    #[test]
    fn unknown_bundle_id_is_none() {
        assert!(MacOsAppIconSource
            .probe_mtime("com.readytype.missing.app")
            .is_none());
        assert!(MacOsAppIconSource
            .fetch("com.readytype.missing.app")
            .is_none());
    }

    #[test]
    fn terminal_bundle_resolves_png() {
        let icon = match MacOsAppIconSource.fetch("com.apple.Terminal") {
            Some(value) => value,
            None => {
                eprintln!("skip: NSWorkspace could not resolve com.apple.Terminal");
                return;
            }
        };
        assert!(!icon.png.is_empty());
        assert!(icon.mtime_secs > 0);
    }
}
