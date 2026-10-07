use crate::domain::input_source::{InputSourceId, PlatformAppId};

use super::input_source_switcher::SwitchDecision;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchLogEntry {
    pub timestamp_unix_ms: u64,
    pub app_id: String,
    pub pid: i32,
    pub generation: u64,
    /// 正規化フォーカス文脈キー（manual override ラッチ単位）。
    pub focus_key: String,
    pub rule_id: String,
    pub decision: SwitchDecision,
    pub from: String,
    pub to: String,
    pub success: bool,
    pub attempted: bool,
    pub reason: String,
    pub detail: String,
    pub processing_micros: u64,
    /// AX スナップショット取得完了からログ記録まで（キュー待ち・gateway 含む）。
    pub pipeline_micros: u64,
    /// ワーカーキュー待ち（AX 取得完了 → prepare_focus 開始）。
    pub queue_micros: u64,
    /// prepare_focus 内のルール判定。
    pub prepare_micros: u64,
    /// NeedsSwitch 返却後〜 gateway/cache ロック取得完了（ロック待ち含む）。Done 経路は 0。
    pub handoff_micros: u64,
    /// gateway.current()（現在の入力ソース取得）。
    pub current_micros: u64,
    /// `current()` がキャッシュヒットしたか。
    pub current_cache_hit: bool,
    /// gateway.select()（未実行なら 0。失敗時の list+再試行を含む）。
    pub select_micros: u64,
    /// complete_focus / 早期 push_log 直前までの後処理。
    pub finalize_micros: u64,
    /// state/tray の publish 所要（pipeline 外。未計測なら 0）。
    pub publish_micros: u64,
    /// AX snapshot_elements 所要（pipeline 外。未計測なら 0）。
    pub ax_micros: u64,
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
