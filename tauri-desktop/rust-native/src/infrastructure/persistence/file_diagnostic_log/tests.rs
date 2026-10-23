use super::*;
use crate::{
    application::{input_source_switcher::SwitchDecision, ports::diagnostic_log::QuickSetLogEntry},
    domain::input_source::{
        AxAttribute, AxElementSnapshot, FocusSnapshot, Platform, PlatformAppId,
    },
};

fn sample_entry(detail: &str) -> SwitchLogEntry {
    SwitchLogEntry {
        timestamp_unix_ms: 1,
        app_id: "com.example".to_owned(),
        pid: 1,
        generation: 2,
        rule_id: "rule".to_owned(),
        decision: SwitchDecision::Switched,
        from: "en".to_owned(),
        to: "jp".to_owned(),
        success: true,
        attempted: true,
        reason: "selected target input source; postSelect=match".to_owned(),
        detail: detail.to_owned(),
        processing_micros: 1,
        prepare_micros: 2,
        current_micros: 3,
        select_micros: 4,
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("readytype-diag-log-{label}-{}", now_unix_ms()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn writes_jsonl_switch_records_with_flag_snapshot() {
    let dir = temp_dir("write");
    let sink = FileDiagnosticLogSink::open(dir.clone()).expect("open");
    sink.append_switch_log(&sample_entry("elements=0"));

    let body = fs::read_to_string(sink.path()).expect("read");
    assert!(body.contains("\"kind\":\"session_start\""));
    assert!(body.contains("\"kind\":\"switch\""));
    assert!(body.contains("\"v\":2"));
    assert!(body.contains("\"decision\":\"switched\""));
    assert!(body.contains("postSelect=match"));
    assert!(body.contains("\"processingMicros\":1"));
    assert!(body.contains("\"prepareMicros\":2"));
    assert!(body.contains("\"currentMicros\":3"));
    assert!(body.contains("\"selectMicros\":4"));
    // schema v2: enabled READYTYPE_* overrides (empty arrays = production defaults)
    assert!(body.contains("\"flags\""));
    assert!(body.contains("\"ablation\""));
    assert!(body.contains("\"proto\""));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn writes_quick_set_launch_and_element_tree() {
    let app = PlatformAppId::new(Platform::MacOs, "com.example.Editor", 42).expect("app");
    let focused = AxElementSnapshot::new()
        .with(AxAttribute::Role, "AXTextField")
        .with(AxAttribute::Identifier, "composer");
    let parent = AxElementSnapshot::new().with(AxAttribute::Role, "AXWindow");
    let snapshot = FocusSnapshot::new(app, 7, vec![focused, parent]);
    let entry =
        QuickSetLogEntry::capture(1_000, "CommandOrControl+Shift+L", "ready", Some(&snapshot));

    let dir = temp_dir("quick-set");
    let sink = FileDiagnosticLogSink::open(dir.clone()).expect("open");
    sink.append_quick_set(&entry);

    let body = fs::read_to_string(sink.path()).expect("read");
    assert!(body.contains("\"kind\":\"quickSet\""));
    assert!(body.contains("\"shortcut\":\"CommandOrControl+Shift+L\""));
    assert!(body.contains("\"outcome\":\"ready\""));
    assert!(body.contains("\"appId\":\"com.example.Editor\""));
    assert!(body.contains("\"depth\":0"));
    assert!(body.contains("\"identifier\":\"composer\""));
    assert!(body.contains("\"depth\":1"));
    assert!(body.contains("\"role\":\"AXWindow\""));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rotates_current_when_file_exceeds_size_limit() {
    let dir = temp_dir("rotate");
    let sink = FileDiagnosticLogSink::open_with_limits(dir.clone(), 800, 50_000).expect("open");

    // 1 行あたり数百バイトあるので、数回で 800 を超えてローテートする。
    for index in 0..12 {
        sink.append_switch_log(&sample_entry(&format!("pad-{}-{}", index, "x".repeat(40))));
    }

    let sessions = fs::read_dir(dir.join("sessions"))
        .expect("sessions")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
        .count();
    assert!(
        sessions >= 1,
        "expected at least one archived session after size rotation"
    );

    let current = fs::read_to_string(dir.join("current.jsonl")).expect("current");
    assert!(current.contains("\"kind\":\"session_start\""));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn prunes_oldest_sessions_when_total_budget_exceeded() {
    let dir = temp_dir("prune");
    fs::create_dir_all(dir.join("sessions")).expect("sessions dir");

    // 起動前に大きな過去セッションを置く（合計予算 2_500 を想定）。
    let payload = "y".repeat(1_200);
    for stamp in ["1000", "2000", "3000"] {
        fs::write(
            dir.join("sessions").join(format!("{stamp}.jsonl")),
            &payload,
        )
        .expect("seed session");
    }

    let _sink = FileDiagnosticLogSink::open_with_limits(dir.clone(), 10_000, 2_500).expect("open");

    let remaining: Vec<_> = fs::read_dir(dir.join("sessions"))
        .expect("sessions")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !remaining.iter().any(|name| name.starts_with("1000")),
        "oldest session should be pruned, remaining={remaining:?}"
    );
    let sessions_bytes: u64 = remaining
        .iter()
        .map(|name| file_len(&dir.join("sessions").join(name)).unwrap_or(0))
        .sum();
    let current_bytes = file_len(&dir.join("current.jsonl")).unwrap_or(0);
    assert!(
        current_bytes.saturating_add(sessions_bytes) <= 2_500,
        "total should fit budget: current={current_bytes} sessions={sessions_bytes}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn flag_snapshot_lists_enabled_proto_and_ablation() {
    let _proto = switch_proto_env_lock();
    let _ablation = ablation_env_lock();
    crate::application::switch_proto::clear_all_proto_env();
    // SAFETY: both env locks held for this process-global mutation.
    unsafe {
        std::env::remove_var(crate::application::switch_ablation::ENV_NO_FOLLOWUP);
        std::env::remove_var(crate::application::switch_ablation::ENV_NO_SAME_SOURCE_RESELECT);
        std::env::remove_var(crate::application::switch_ablation::ENV_NO_RETRY);
        std::env::set_var(crate::application::switch_proto::ENV_DRAIN_FOCUS, "1");
        std::env::set_var(crate::application::switch_ablation::ENV_NO_RETRY, "1");
    }

    let snapshot = FlagSnapshot::capture();
    assert_eq!(snapshot.proto, vec!["drainFocus"]);
    assert_eq!(snapshot.ablation, vec!["noRetry"]);

    unsafe {
        std::env::remove_var(crate::application::switch_proto::ENV_DRAIN_FOCUS);
        std::env::remove_var(crate::application::switch_ablation::ENV_NO_RETRY);
    }
}

fn switch_proto_env_lock() -> std::sync::MutexGuard<'static, ()> {
    crate::application::switch_proto::PROTO_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn ablation_env_lock() -> std::sync::MutexGuard<'static, ()> {
    // Reuse proto lock ordering: always take PROTO then a dedicated ablation lock.
    crate::application::switch_ablation::ABLATION_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
