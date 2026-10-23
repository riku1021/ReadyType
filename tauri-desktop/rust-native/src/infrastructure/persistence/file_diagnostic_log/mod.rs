//! プロジェクト内 `tmp/input-source-logs/` へ切替診断を JSONL 追記する。
//!
//! 構成（Git 対象外）:
//! ```text
//! tmp/input-source-logs/
//!   current.jsonl                 # いまのセグメント（起動時・サイズ超過でローテート）
//!   sessions/<unixSecs>.jsonl     # 過去セグメント
//! ```
//!
//! 保持:
//! - 1 ファイル最大約 10 MiB でローテート（常駐プロセス向け）
//! - `current` + `sessions` の合計約 50 MiB を超えたら古いセッションから削除
//! - セッション件数の上限 30（サイズ上限の補助）
//!
//! 上書き: 環境変数 `READYTYPE_INPUT_SOURCE_LOG_DIR`

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::application::{
    input_source_switcher::SwitchDecision,
    ports::diagnostic_log::{DiagnosticLogSink, QuickSetElementLog, QuickSetLogEntry},
    switch_ablation, switch_proto,
    switcher_state::SwitchLogEntry,
};

const SCHEMA_VERSION: u32 = 2;
/// 1 本の JSONL がこのサイズ以上になったら `sessions/` へ退避する。
const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;
/// `current` + `sessions` の合計上限。
const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;
/// 小さなセッションが大量に残る場合の件数上限（サイズ上限の補助）。
const MAX_SESSION_FILES: usize = 30;

pub struct FileDiagnosticLogSink {
    path: PathBuf,
    state: Mutex<SinkState>,
}

struct SinkState {
    dir: PathBuf,
    file: Option<File>,
    bytes: u64,
    max_file_bytes: u64,
    max_total_bytes: u64,
}

impl FileDiagnosticLogSink {
    /// リポジトリ直下の `tmp/input-source-logs` を開き、前回 `current.jsonl` を sessions へ退避する。
    pub fn open_default() -> Result<Self, String> {
        Self::open(resolve_log_dir()?)
    }

    pub fn open(dir: PathBuf) -> Result<Self, String> {
        Self::open_with_limits(dir, MAX_FILE_BYTES, MAX_TOTAL_BYTES)
    }

    fn open_with_limits(
        dir: PathBuf,
        max_file_bytes: u64,
        max_total_bytes: u64,
    ) -> Result<Self, String> {
        fs::create_dir_all(dir.join("sessions"))
            .map_err(|error| format!("create log dir: {error}"))?;
        rotate_current_into_sessions(&dir)?;

        let path = dir.join("current.jsonl");
        let (file, bytes) = open_new_current(&path)?;
        // current 作成後に合計を見る（session_start 分を含める）。
        prune_log_budget(&dir, max_total_bytes)?;

        Ok(Self {
            path,
            state: Mutex::new(SinkState {
                dir,
                file: Some(file),
                bytes,
                max_file_bytes,
                max_total_bytes,
            }),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl DiagnosticLogSink for FileDiagnosticLogSink {
    fn append_switch_log(&self, entry: &SwitchLogEntry) {
        let record = SwitchLogFileRecord::from_entry(entry);
        let Ok(line) = serde_json::to_string(&record) else {
            return;
        };
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let _ = state.append_line(&line);
    }

    fn append_quick_set(&self, entry: &QuickSetLogEntry) {
        let record = QuickSetFileRecord::from_entry(entry);
        let Ok(line) = serde_json::to_string(&record) else {
            return;
        };
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let _ = state.append_line(&line);
    }
}

impl SinkState {
    fn append_line(&mut self, line: &str) -> Result<(), String> {
        let line_bytes = u64::try_from(line.len().saturating_add(1)).unwrap_or(u64::MAX);
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| "diagnostic log file is closed".to_owned())?;
        writeln!(file, "{line}").map_err(|error| format!("write log: {error}"))?;
        file.flush()
            .map_err(|error| format!("flush log: {error}"))?;
        self.bytes = self.bytes.saturating_add(line_bytes);
        // 書き込み後に上限を超えていれば次の追記に備えてローテートする。
        // （先に判定すると session_start + 1 行で上限を超える場合に無限ローテートし得る）
        if self.bytes >= self.max_file_bytes {
            self.rotate()?;
        }
        Ok(())
    }

    fn rotate(&mut self) -> Result<(), String> {
        self.file = None;
        rotate_current_into_sessions(&self.dir)?;
        let path = self.dir.join("current.jsonl");
        let (file, bytes) = open_new_current(&path)?;
        prune_log_budget(&self.dir, self.max_total_bytes)?;
        self.file = Some(file);
        self.bytes = bytes;
        Ok(())
    }
}

fn open_new_current(path: &Path) -> Result<(File, u64), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|error| format!("open {}: {error}", path.display()))?;

    let start = SessionMetaRecord {
        v: SCHEMA_VERSION,
        kind: "session_start",
        timestamp_unix_ms: now_unix_ms(),
        path: path.display().to_string(),
        note: "ReadyType input-source switch diagnostics (JSONL). One object per line. Prefer current.jsonl. flags.ablation / flags.proto list env overrides that are ON (empty = production defaults).",
        flags: FlagSnapshot::capture(),
    };
    let line = serde_json::to_string(&start).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| format!("write session_start: {error}"))?;
    file.flush()
        .map_err(|error| format!("flush session_start: {error}"))?;
    let bytes = u64::try_from(line.len().saturating_add(1)).unwrap_or(u64::MAX);
    Ok((file, bytes))
}

/// Enabled `READYTYPE_ABLATION_*` / `READYTYPE_PROTO_*` at write time.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct FlagSnapshot {
    ablation: Vec<&'static str>,
    proto: Vec<&'static str>,
}

impl FlagSnapshot {
    fn capture() -> Self {
        Self {
            ablation: switch_ablation::active_slugs(),
            proto: switch_proto::active_slugs(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionMetaRecord {
    v: u32,
    kind: &'static str,
    timestamp_unix_ms: u64,
    path: String,
    note: &'static str,
    flags: FlagSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SwitchLogFileRecord<'a> {
    v: u32,
    kind: &'static str,
    timestamp_unix_ms: u64,
    app_id: &'a str,
    pid: i32,
    generation: u64,
    rule_id: &'a str,
    decision: &'static str,
    from: &'a str,
    to: &'a str,
    success: bool,
    attempted: bool,
    reason: &'a str,
    detail: &'a str,
    processing_micros: u64,
    prepare_micros: u64,
    current_micros: u64,
    select_micros: u64,
    flags: FlagSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuickSetFileRecord<'a> {
    v: u32,
    kind: &'static str,
    timestamp_unix_ms: u64,
    shortcut: &'a str,
    outcome: &'a str,
    app_id: &'a str,
    pid: i32,
    generation: u64,
    elements: Vec<QuickSetElementRecord<'a>>,
    flags: FlagSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuickSetElementRecord<'a> {
    depth: u32,
    #[serde(skip_serializing_if = "str::is_empty")]
    role: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    subrole: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    role_description: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    title: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    description: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    identifier: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    dom_identifier: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    dom_class_list: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    help: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    placeholder: &'a str,
}

impl<'a> QuickSetFileRecord<'a> {
    fn from_entry(entry: &'a QuickSetLogEntry) -> Self {
        Self {
            v: SCHEMA_VERSION,
            kind: "quickSet",
            timestamp_unix_ms: entry.timestamp_unix_ms,
            shortcut: entry.shortcut.as_str(),
            outcome: entry.outcome.as_str(),
            app_id: entry.app_id.as_str(),
            pid: entry.pid,
            generation: entry.generation,
            elements: entry
                .elements
                .iter()
                .map(QuickSetElementRecord::from)
                .collect(),
            flags: FlagSnapshot::capture(),
        }
    }
}

impl<'a> From<&'a QuickSetElementLog> for QuickSetElementRecord<'a> {
    fn from(element: &'a QuickSetElementLog) -> Self {
        Self {
            depth: element.depth,
            role: element.role.as_str(),
            subrole: element.subrole.as_str(),
            role_description: element.role_description.as_str(),
            title: element.title.as_str(),
            description: element.description.as_str(),
            identifier: element.identifier.as_str(),
            dom_identifier: element.dom_identifier.as_str(),
            dom_class_list: element.dom_class_list.as_str(),
            help: element.help.as_str(),
            placeholder: element.placeholder.as_str(),
        }
    }
}

impl<'a> SwitchLogFileRecord<'a> {
    fn from_entry(entry: &'a SwitchLogEntry) -> Self {
        Self {
            v: SCHEMA_VERSION,
            kind: "switch",
            timestamp_unix_ms: entry.timestamp_unix_ms,
            app_id: entry.app_id.as_str(),
            pid: entry.pid,
            generation: entry.generation,
            rule_id: entry.rule_id.as_str(),
            decision: decision_slug(entry.decision),
            from: entry.from.as_str(),
            to: entry.to.as_str(),
            success: entry.success,
            attempted: entry.attempted,
            reason: entry.reason.as_str(),
            detail: entry.detail.as_str(),
            processing_micros: entry.processing_micros,
            prepare_micros: entry.prepare_micros,
            current_micros: entry.current_micros,
            select_micros: entry.select_micros,
            flags: FlagSnapshot::capture(),
        }
    }
}

fn decision_slug(decision: SwitchDecision) -> &'static str {
    match decision {
        SwitchDecision::Ignored => "ignored",
        SwitchDecision::Disabled => "disabled",
        SwitchDecision::Snoozed => "snoozed",
        SwitchDecision::NoRule => "noRule",
        SwitchDecision::AlreadySelected => "alreadySelected",
        SwitchDecision::Switched => "switched",
        SwitchDecision::Failed => "failed",
    }
}

fn resolve_log_dir() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("READYTYPE_INPUT_SOURCE_LOG_DIR") {
        let path = PathBuf::from(dir);
        if path.as_os_str().is_empty() {
            return Err("READYTYPE_INPUT_SOURCE_LOG_DIR is empty".to_owned());
        }
        return Ok(path);
    }

    let mut starts = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    starts.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    for start in starts {
        if let Some(root) = find_repo_root(&start) {
            return Ok(root.join("tmp/input-source-logs"));
        }
    }

    let fallback = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("tmp/input-source-logs");
    Ok(fallback)
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let has_git = dir.join(".git").exists();
        let looks_like_readytype = dir.join("tauri-desktop").is_dir() && dir.join("docs").is_dir();
        if has_git || looks_like_readytype {
            return Some(dir.to_path_buf());
        }
    }
    None
}

fn rotate_current_into_sessions(dir: &Path) -> Result<(), String> {
    let current = dir.join("current.jsonl");
    if !current.is_file() {
        return Ok(());
    }
    let stamp = now_unix_ms() / 1000;
    let archived = dir.join("sessions").join(format!("{stamp}.jsonl"));
    // 同一秒に二重起動・サイズローテートした場合はサフィックスを付ける。
    let archived = if archived.exists() {
        dir.join("sessions")
            .join(format!("{stamp}-{}.jsonl", now_unix_ms() % 1000))
    } else {
        archived
    };
    fs::rename(&current, &archived).map_err(|error| {
        format!(
            "rotate {} -> {}: {error}",
            current.display(),
            archived.display()
        )
    })
}

fn prune_log_budget(dir: &Path, max_total_bytes: u64) -> Result<(), String> {
    let sessions_dir = dir.join("sessions");
    let mut session_entries = fs::read_dir(&sessions_dir)
        .map_err(|error| format!("read sessions: {error}"))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
        .collect::<Vec<_>>();

    // ファイル名は unix 秒ベース。古いものから削除する。
    session_entries.sort_by_key(|entry| entry.file_name());

    let current_bytes = file_len(&dir.join("current.jsonl")).unwrap_or(0);
    let mut sessions_bytes: u64 = session_entries
        .iter()
        .map(|entry| file_len(&entry.path()).unwrap_or(0))
        .sum();

    while session_entries.len() > MAX_SESSION_FILES
        || current_bytes.saturating_add(sessions_bytes) > max_total_bytes
    {
        let Some(oldest) = session_entries.first() else {
            break;
        };
        let path = oldest.path();
        let removed = file_len(&path).unwrap_or(0);
        let _ = fs::remove_file(&path);
        session_entries.remove(0);
        sessions_bytes = sessions_bytes.saturating_sub(removed);
    }
    Ok(())
}

fn file_len(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|meta| meta.len())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
