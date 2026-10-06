use std::sync::Arc;

use crate::domain::input_source::FocusSnapshot;

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

pub type FocusEventSink = Arc<dyn Fn(FocusSnapshot, u64) + Send + Sync>;

/// アプリ bundle id → 切替判定に必要な AX 属性。
pub type SwitchingAttributeIndex = Vec<(String, Vec<crate::domain::input_source::AxAttribute>)>;

pub trait FocusMonitor: Send {
    fn start(
        &mut self,
        attribute_index: SwitchingAttributeIndex,
        sink: FocusEventSink,
    ) -> Result<(), FocusMonitorError>;
    fn stop(&mut self) -> Result<(), FocusMonitorError>;
    fn set_debug_capture(&mut self, enabled: bool) -> Result<(), FocusMonitorError>;
    fn permission_status(&self) -> AccessibilityPermission;
    fn request_permission(&self) -> Result<AccessibilityPermission, FocusMonitorError>;
    fn open_settings(&self) -> Result<(), FocusMonitorError>;
}
