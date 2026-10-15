//! Outbound port for resolving OS application icons.

/// Cached PNG path plus a revision token for WebView cache busting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAppIcon {
    /// Absolute filesystem path to a cached PNG.
    pub path: String,
    /// Changes when the on-disk icon bytes should be considered new
    /// (`{mtimeSecs}:{styleToken}`).
    pub revision: String,
}

/// Resolves an application icon for a platform bundle identifier.
/// Implementations live in `infrastructure::platform::{macos,…}`.
pub trait AppIconResolver: Send + Sync {
    /// Returns a cached PNG path + revision for the bundle id,
    /// or `None` when the application/icon cannot be resolved.
    fn resolve(&self, bundle_id: &str) -> Option<ResolvedAppIcon>;
}
