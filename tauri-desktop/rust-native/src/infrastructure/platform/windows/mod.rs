//! Windows-specific infrastructure adapters.

use crate::application::ports::platform::PlatformGateway;

pub struct WindowsPlatform;

impl PlatformGateway for WindowsPlatform {
    fn display_name(&self) -> &'static str {
        "windows"
    }
}
