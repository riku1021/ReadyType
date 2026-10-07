use crate::application::ports::current_input_source_change::{
    CurrentInputSourceChangeMonitor, CurrentInputSourceChangeSink,
};

/// 非 macOS / 未対応環境用の no-op。
pub struct UnsupportedCurrentInputSourceChangeMonitor;

impl CurrentInputSourceChangeMonitor for UnsupportedCurrentInputSourceChangeMonitor {
    fn start(&mut self, _sink: CurrentInputSourceChangeSink) -> Result<(), String> {
        Ok(())
    }

    fn stop(&mut self) {}
}
