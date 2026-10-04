//! Outbound port for resolving OS application display names.

/// Resolves a user-visible application name for a platform bundle identifier.
/// Implementations live in `infrastructure::platform::{macos,…}`.
pub trait AppDisplayNameResolver: Send + Sync {
    /// Returns the Finder-style display name for the bundle id,
    /// or `None` when the application cannot be resolved.
    fn resolve(&self, bundle_id: &str) -> Option<String>;
}
