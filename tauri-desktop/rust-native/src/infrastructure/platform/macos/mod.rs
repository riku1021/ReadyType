//! macOS-specific infrastructure adapters.

mod window_chrome;

pub use window_chrome::set_window_background_main;

use crate::application::ports::platform::PlatformGateway;

pub struct MacOsPlatform;

impl PlatformGateway for MacOsPlatform {
    fn display_name(&self) -> &'static str {
        "macos"
    }
}
