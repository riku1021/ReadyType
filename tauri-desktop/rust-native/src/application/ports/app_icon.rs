//! Outbound port for resolving OS application icons.

/// Resolves an application icon for a platform bundle identifier.
/// Implementations live in `infrastructure::platform::{macos,…}`.
pub trait AppIconResolver: Send + Sync {
    /// Returns a PNG data URL (`data:image/png;base64,…`) for the bundle id,
    /// or `None` when the application/icon cannot be resolved.
    fn resolve(&self, bundle_id: &str) -> Option<String>;
}
