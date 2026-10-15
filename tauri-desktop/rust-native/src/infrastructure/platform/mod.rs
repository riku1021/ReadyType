//! OS-specific adapters implementing `application::ports::platform::PlatformGateway`.

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOsPlatform as CurrentPlatform;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform as CurrentPlatform;

pub mod app_display_name;
pub mod app_display_name_cache;
pub mod app_icon;
pub mod app_icon_cache;
pub mod focus_monitor;
pub mod input_source;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform as CurrentPlatform;
