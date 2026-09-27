//! macOS-specific infrastructure adapters.

mod accessibility;
mod app_icon;
mod focus_monitor;
mod tis;
mod window_chrome;

pub use app_icon::MacOsAppIconSource;
pub use focus_monitor::MacOsFocusMonitor;
pub use tis::SystemTisBackend;
pub use window_chrome::set_window_background_main;

use crate::application::ports::platform::PlatformGateway;

pub struct MacOsPlatform;

impl PlatformGateway for MacOsPlatform {
    fn display_name(&self) -> &'static str {
        "macos"
    }
}
