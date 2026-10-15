//! プロジェクト内 `tmp/input-source-logs/` へ切替診断を JSONL 追記する。
//!
//! 構成（Git 対象外）:
//! ```text
//! tmp/input-source-logs/
//!   current.jsonl                 # いまのセッション（起動のたびにローテート）
//!   sessions/<unixSecs>.jsonl     # 過去セッション（最大 30）
//! ```
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
    input_source_switcher::SwitchDecision, ports::diagnostic_log::DiagnosticLogSink,
    switch_ablation, switch_proto, switcher_state::SwitchLogEntry,
};

const SCHEMA_VERSION: u32 = 2;
const MAX_SESSION_FILES: usize = 30;

pub struct FileDiagnosticLogSink {
    file: Mutex<File>,
    path: PathBuf,
}

impl FileDiagnosticLogSink {
    /// リポジトリ直下の `tmp/input-source-logs` を開き、前回 `current.jsonl` を sessions へ退避する。
    pub fn open_default() -> Result<Self, String> {
        Self::open(resolve_log_dir()?)
    }

    pub fn open(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(dir.join("sessions"))
            .map_err(|error| format!("create log dir: {error}"))?;
        rotate_current_into_sessions(&dir)?;
        prune_old_sessions(&dir.join("sessions"), MAX_SESSION_FILES)?;

        let path = dir.join("current.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|error| format!("open {}: {error}", path.display()))?;

        let start = SessionMetaRecord {
            v: SCHEMA_VERSION,
            kind: "session_start",
            timestamp_unix_ms: now_unix_ms(),
            path: path.display().to_string(),
            note: "ReadyType input-source switch diagnostics (JSONL). One object per line. Prefer current.jsonl. flags.ablation / flags.proto list env overrides that are ON (empty = production defaults).",
            flags: FlagSnapshot::capture(),
        };
        writeln!(
            file,
            "{}",
            serde_json::to_string(&start).map_err(|error| error.to_string())?
        )
        .map_err(|error| format!("write session_start: {error}"))?;

        Ok(Self {
            file: Mutex::new(file),
            path,
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
        let Ok(mut file) = self.file.lock() else {
            return;
        };
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
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
    // 同一秒に二重起動した場合はサフィックスを付ける。
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

fn prune_old_sessions(sessions_dir: &Path, keep: usize) -> Result<(), String> {
    let mut entries = fs::read_dir(sessions_dir)
        .map_err(|error| format!("read sessions: {error}"))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
    for entry in entries.into_iter().skip(keep) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
