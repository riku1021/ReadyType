//! macOS application icon source backed by `NSWorkspace`.

use objc2::AnyThread;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
use objc2_foundation::{NSDictionary, NSString};

use crate::infrastructure::platform::app_icon_cache::{
    file_mtime_secs, AppIconSource, FetchedIcon,
};
use crate::infrastructure::platform::macos::application_path;

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
mod tests;
