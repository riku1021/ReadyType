use std::sync::Arc;

use crate::domain::input_source::{AxAttribute, FocusSnapshot, PlatformAppId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityPermission {
    Granted,
    Denied,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusMonitorError {
    Operation(String),
    UnsupportedPlatform,
}

pub type FocusEventSink = Arc<dyn Fn(FocusSnapshot) + Send + Sync>;

pub trait FocusMonitor: Send {
    fn start(
        &mut self,
        attributes: Vec<AxAttribute>,
        sink: FocusEventSink,
    ) -> Result<(), FocusMonitorError>;
    fn stop(&mut self) -> Result<(), FocusMonitorError>;
    fn set_debug_capture(&mut self, enabled: bool) -> Result<(), FocusMonitorError>;
    fn permission_status(&self) -> AccessibilityPermission;
    fn request_permission(&self) -> Result<AccessibilityPermission, FocusMonitorError>;
    fn open_settings(&self) -> Result<(), FocusMonitorError>;
    /// Returns the foreground window's explicit AUMID when it belongs to this process.
    fn foreground_app_user_model_id(&self, _process_id: i32) -> Option<String> {
        None
    }
    /// 現在の前面アプリ（Quick Set の stale snapshot ガード用）。
    fn frontmost_app(&self) -> Option<PlatformAppId> {
        None
    }
}
