//! macOS-specific infrastructure adapters.

mod accessibility;
mod app_display_name;
mod app_icon;
mod application_path;
mod current_input_source_change;
mod focus_monitor;
mod input_source_display_name;
pub(crate) mod lab_switch_window;
mod tis;
mod window_chrome;

pub use current_input_source_change::MacOsCurrentInputSourceChangeMonitor;

pub use accessibility::{focused_element_frame, ElementFrame};
pub use app_display_name::MacOsAppDisplayNameResolver;
pub use app_icon::MacOsAppIconSource;
pub use application_path::application_path;
pub use focus_monitor::MacOsFocusMonitor;
pub use tis::{inspect_input_source_flags, set_followup_reselect_log_sink, SystemTisBackend};
pub use window_chrome::{set_window_background_main, set_window_background_transparent};

use crate::application::ports::platform::PlatformGateway;

pub struct MacOsPlatform;

impl PlatformGateway for MacOsPlatform {
    fn display_name(&self) -> &'static str {
        "macos"
    }
}
