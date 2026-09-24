//! OS / platform capabilities exposed to the application layer.

/// Outbound port for OS-specific behavior.
/// Implement in `infrastructure::platform::{macos,windows,linux}`.
pub trait PlatformGateway {
    /// Human-readable OS label for diagnostics and demos.
    fn display_name(&self) -> &'static str;
}
