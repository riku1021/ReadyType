//! Port for OS notifications when the selected keyboard input source changes.

use std::sync::Arc;

/// Callback invoked when the OS reports a selected-input-source change.
pub type CurrentInputSourceChangeSink = Arc<dyn Fn() + Send + Sync>;

pub trait CurrentInputSourceChangeMonitor: Send {
    fn start(&mut self, sink: CurrentInputSourceChangeSink) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
}

/// No-op monitor used when PROTO_TIS_NOTIFY is off or platform unsupported.
#[derive(Default)]
pub struct NoopCurrentInputSourceChangeMonitor;

impl CurrentInputSourceChangeMonitor for NoopCurrentInputSourceChangeMonitor {
    fn start(&mut self, _sink: CurrentInputSourceChangeSink) -> Result<(), String> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
}
