//! Platform-independent fallback for app display name resolution.

use crate::application::ports::app_display_name::AppDisplayNameResolver;

/// Resolver used on platforms without a display-name implementation. Always returns `None`.
pub struct UnsupportedAppDisplayNameResolver;

impl AppDisplayNameResolver for UnsupportedAppDisplayNameResolver {
    fn resolve(&self, _bundle_id: &str) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests;
