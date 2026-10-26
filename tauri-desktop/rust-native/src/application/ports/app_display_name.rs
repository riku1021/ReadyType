//! Outbound port for resolving OS application display names.

/// Resolves a user-visible application name for a platform bundle identifier.
/// Implementations live in `infrastructure::platform::{macos,…}`.
pub trait AppDisplayNameResolver: Send + Sync {
    /// Returns the Finder-style display name for the bundle id,
    /// or `None` when the application cannot be resolved.
    fn resolve(&self, bundle_id: &str) -> Option<String>;

    /// Whether bundle-only resolutions stay stable for the lifetime of the resolver.
    fn is_bundle_resolution_cacheable(&self) -> bool {
        true
    }

    /// Resolves a name when the caller has process context, if the platform needs it.
    fn resolve_for_process(&self, bundle_id: &str, _process_id: i32) -> Option<String> {
        self.resolve(bundle_id)
    }

    /// Resolves with window metadata when Windows exposes an explicit AUMID.
    fn resolve_for_process_with_aumid(
        &self,
        bundle_id: &str,
        process_id: i32,
        _app_user_model_id: Option<&str>,
    ) -> Option<String> {
        self.resolve_for_process(bundle_id, process_id)
    }
}
