//! Platform-independent fallback for app icon resolution.

use crate::application::ports::app_icon::AppIconResolver;
use crate::infrastructure::platform::app_icon_cache::{AppIconSource, FetchedIcon};

/// Source used on platforms without an icon implementation. Always returns `None`.
pub struct UnsupportedAppIconSource;

impl AppIconSource for UnsupportedAppIconSource {
    fn probe_mtime(&self, _bundle_id: &str) -> Option<u64> {
        None
    }

    fn fetch(&self, _bundle_id: &str) -> Option<FetchedIcon> {
        None
    }
}

/// Resolver used on platforms without an icon implementation. Always returns `None`.
pub struct UnsupportedAppIconResolver;

impl AppIconResolver for UnsupportedAppIconResolver {
    fn resolve(&self, _bundle_id: &str) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests;
