//! macOS-specific infrastructure adapters.

mod accessibility;
mod accessibility_permission_monitor;
mod app_display_name;
mod app_icon;
mod application_path;
mod current_input_source_change;
mod enabled_input_sources_monitor;
mod focus_monitor;
mod icon_style;
mod input_source_display_name;
pub(crate) mod lab_switch_window;
mod tis;
mod window_chrome;

pub use current_input_source_change::MacOsCurrentInputSourceChangeMonitor;

pub use accessibility::{focused_element_frame, ElementFrame};
pub use accessibility_permission_monitor::start_monitor as start_accessibility_permission_monitor;
pub use app_display_name::MacOsAppDisplayNameResolver;
pub use app_icon::MacOsAppIconSource;
pub use application_path::application_path;
pub use enabled_input_sources_monitor::start_monitor as start_enabled_input_sources_monitor;
pub use focus_monitor::MacOsFocusMonitor;
pub use icon_style::{
    shared as macos_icon_style_source, start_monitor as start_icon_style_monitor,
};
pub use tis::{inspect_input_source_flags, set_followup_reselect_log_sink, SystemTisBackend};
pub use window_chrome::{set_window_background_main, set_window_background_transparent};

use crate::application::ports::platform::PlatformGateway;

pub struct MacOsPlatform;

impl PlatformGateway for MacOsPlatform {
    fn display_name(&self) -> &'static str {
        "macos"
    }
}
