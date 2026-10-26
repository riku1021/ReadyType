//! Windows-specific infrastructure adapters.

mod app_display_name;
mod current_input_source_change;
mod focus_monitor;
pub(crate) mod input_source;

pub use app_display_name::WindowsAppDisplayNameResolver;
pub use current_input_source_change::WindowsCurrentInputSourceChangeMonitor;
pub use focus_monitor::WindowsFocusMonitor;

use crate::application::ports::platform::PlatformGateway;

pub struct WindowsPlatform;

impl PlatformGateway for WindowsPlatform {
    fn display_name(&self) -> &'static str {
        "windows"
    }
}
