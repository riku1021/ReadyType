//! Linux-specific infrastructure adapters.

use crate::application::ports::platform::PlatformGateway;

pub struct LinuxPlatform;

impl PlatformGateway for LinuxPlatform {
    fn display_name(&self) -> &'static str {
        "linux"
    }
}
