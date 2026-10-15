use crate::domain::input_source::{InputSourceId, PlatformAppId};

use super::input_source_switcher::SwitchDecision;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchLogEntry {
    pub timestamp_unix_ms: u64,
    pub app_id: String,
    pub pid: i32,
    pub generation: u64,
    pub rule_id: String,
    pub decision: SwitchDecision,
    pub from: String,
    pub to: String,
    pub success: bool,
    pub attempted: bool,
    pub reason: String,
    pub detail: String,
    pub processing_micros: u64,
    /// `prepare_focus` 区間（ルール判定）。NeedsSwitch 時は complete 前までの時間。
    pub prepare_micros: u64,
    /// `gateway.current()` 区間。
    pub current_micros: u64,
    /// `gateway.select`（retry 含む）。未実行時 0。
    pub select_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitcherState {
    pub version: u64,
    pub rules_version: u64,
    pub enabled: bool,
    pub snoozed_until_unix_ms: Option<u64>,
    pub current_app: Option<PlatformAppId>,
    pub matched_rule_id: Option<String>,
    pub current_input_source: Option<InputSourceId>,
    pub target_input_source: Option<InputSourceId>,
    pub last_decision: Option<SwitchDecision>,
    pub logs: Vec<SwitchLogEntry>,
}

impl SwitcherState {
    pub fn new(enabled: bool, snoozed_until_unix_ms: Option<u64>) -> Self {
        Self {
            version: 0,
            rules_version: 1,
            enabled,
            snoozed_until_unix_ms,
            current_app: None,
            matched_rule_id: None,
            current_input_source: None,
            target_input_source: None,
            last_decision: None,
            logs: Vec::new(),
        }
    }
}
