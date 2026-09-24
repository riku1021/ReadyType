use crate::domain::input_source::{InputSourceId, PlatformAppId};

use super::input_source_switcher::SwitchDecision;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchLogEntry {
    pub timestamp_unix_ms: u64,
    pub app_id: String,
    pub rule_id: String,
    pub from: InputSourceId,
    pub to: InputSourceId,
    pub success: bool,
    pub reason: String,
    pub processing_micros: u64,
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
