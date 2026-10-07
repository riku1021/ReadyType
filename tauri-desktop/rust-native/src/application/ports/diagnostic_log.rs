//! 診断ログの永続化（ファイル等）。UI の in-memory ログと並行して書く。

use std::sync::Arc;

use crate::application::switcher_state::SwitchLogEntry;

pub trait DiagnosticLogSink: Send + Sync {
    fn append_switch_log(&self, entry: &SwitchLogEntry);
}

pub type SharedDiagnosticLogSink = Arc<dyn DiagnosticLogSink>;

/// 何もしない（テスト / sink 未配線時）。
pub struct NoopDiagnosticLogSink;

impl DiagnosticLogSink for NoopDiagnosticLogSink {
    fn append_switch_log(&self, _entry: &SwitchLogEntry) {}
}
