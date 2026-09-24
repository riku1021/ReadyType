use std::sync::Arc;

use crate::domain::input_source::{AxAttribute, FocusSnapshot};

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
}
