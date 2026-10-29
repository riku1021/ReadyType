//! 診断ログの永続化（ファイル等）。UI の in-memory ログと並行して書く。

use std::sync::Arc;

use crate::{
    application::switcher_state::SwitchLogEntry,
    domain::input_source::{AxAttribute, FocusSnapshot},
};

pub trait DiagnosticLogSink: Send + Sync {
    fn append_switch_log(&self, entry: &SwitchLogEntry);

    /// Startup stages and blocking platform calls.
    fn append_startup_stage(&self, _entry: &StartupStageLogEntry) {}

    /// Quick Set ショートカット起動と、そのとき使った AX ツリー。
    /// 既定は何もしない（切替ログだけ実装したテストダブル向け）。
    fn append_quick_set(&self, _entry: &QuickSetLogEntry) {}

    /// Quick Set 起動処理の段階ログ。実装しない sink では何もしない。
    fn append_quick_set_stage(&self, _entry: &QuickSetStageLogEntry) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupStageLogEntry {
    pub timestamp_unix_ms: u64,
    pub stage: String,
    pub outcome: String,
    pub elapsed_micros: u64,
    pub error: Option<String>,
}

impl StartupStageLogEntry {
    pub fn new(stage: &str, outcome: &str, elapsed_micros: u64, error: Option<&str>) -> Self {
        let timestamp_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| {
                duration.as_millis().try_into().unwrap_or(u64::MAX)
            });
        Self {
            timestamp_unix_ms,
            stage: stage.to_owned(),
            outcome: outcome.to_owned(),
            elapsed_micros,
            error: error.map(str::to_owned),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickSetStageLogEntry {
    pub timestamp_unix_ms: u64,
    pub shortcut: String,
    pub stage: String,
    pub outcome: String,
    pub elapsed_micros: u64,
    pub error: Option<String>,
    pub app_id: String,
    pub pid: i32,
    pub generation: u64,
}

impl QuickSetStageLogEntry {
    pub fn new(
        timestamp_unix_ms: u64,
        shortcut: &str,
        stage: &str,
        outcome: &str,
        elapsed_micros: u64,
        error: Option<&str>,
        snapshot: Option<&FocusSnapshot>,
    ) -> Self {
        let (app_id, pid, generation) = snapshot.map_or_else(
            || (String::new(), 0, 0),
            |snapshot| {
                (
                    snapshot.app().app_id().to_owned(),
                    snapshot.app().pid(),
                    snapshot.generation(),
                )
            },
        );
        Self {
            timestamp_unix_ms,
            shortcut: shortcut.to_owned(),
            stage: stage.to_owned(),
            outcome: outcome.to_owned(),
            elapsed_micros,
            error: error.map(str::to_owned),
            app_id,
            pid,
            generation,
        }
    }
}

/// ショートカット起動 1 回分。要素 0 がフォーカス、以降が祖先。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickSetLogEntry {
    pub timestamp_unix_ms: u64,
    pub shortcut: String,
    pub outcome: String,
    pub app_id: String,
    pub pid: i32,
    pub generation: u64,
    pub elements: Vec<QuickSetElementLog>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickSetElementLog {
    pub depth: u32,
    pub role: String,
    pub subrole: String,
    pub role_description: String,
    pub title: String,
    pub description: String,
    pub identifier: String,
    pub dom_identifier: String,
    pub dom_class_list: String,
    pub help: String,
    pub placeholder: String,
}

impl QuickSetLogEntry {
    pub fn capture(
        timestamp_unix_ms: u64,
        shortcut: &str,
        outcome: &str,
        snapshot: Option<&FocusSnapshot>,
    ) -> Self {
        let Some(snapshot) = snapshot else {
            return Self {
                timestamp_unix_ms,
                shortcut: shortcut.to_owned(),
                outcome: outcome.to_owned(),
                app_id: String::new(),
                pid: 0,
                generation: 0,
                elements: Vec::new(),
            };
        };
        let elements = snapshot
            .elements()
            .iter()
            .enumerate()
            .map(|(depth, element)| QuickSetElementLog {
                depth: u32::try_from(depth).unwrap_or(u32::MAX),
                role: element.value(AxAttribute::Role).to_owned(),
                subrole: element.value(AxAttribute::Subrole).to_owned(),
                role_description: element.value(AxAttribute::RoleDescription).to_owned(),
                title: element.value(AxAttribute::Title).to_owned(),
                description: element.value(AxAttribute::Description).to_owned(),
                identifier: element.value(AxAttribute::Identifier).to_owned(),
                dom_identifier: element.value(AxAttribute::DomIdentifier).to_owned(),
                dom_class_list: element.value(AxAttribute::DomClassList).to_owned(),
                help: element.value(AxAttribute::Help).to_owned(),
                placeholder: element.value(AxAttribute::Placeholder).to_owned(),
            })
            .collect();
        Self {
            timestamp_unix_ms,
            shortcut: shortcut.to_owned(),
            outcome: outcome.to_owned(),
            app_id: snapshot.app().app_id().to_owned(),
            pid: snapshot.app().pid(),
            generation: snapshot.generation(),
            elements,
        }
    }
}

pub type SharedDiagnosticLogSink = Arc<dyn DiagnosticLogSink>;

/// 何もしない（テスト / sink 未配線時）。
pub struct NoopDiagnosticLogSink;

impl DiagnosticLogSink for NoopDiagnosticLogSink {
    fn append_switch_log(&self, _entry: &SwitchLogEntry) {}
}
