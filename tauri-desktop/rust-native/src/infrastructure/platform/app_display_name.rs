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
mod tests {
    use super::*;

    #[test]
    fn unresolved_bundle_ids_are_none() {
        let resolver = UnsupportedAppDisplayNameResolver;
        for bundle_id in ["", "com.unknown.app", "com.apple.Terminal"] {
            assert_eq!(resolver.resolve(bundle_id), None, "bundle_id={bundle_id:?}");
        }
    }
}
