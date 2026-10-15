use super::*;
use crate::application::input_source_switcher::SwitchDecision;

#[test]
fn writes_jsonl_switch_records_with_flag_snapshot() {
    let dir = std::env::temp_dir().join(format!("readytype-diag-log-{}", now_unix_ms()));
    let _ = fs::remove_dir_all(&dir);
    let sink = FileDiagnosticLogSink::open(dir.clone()).expect("open");
    sink.append_switch_log(&SwitchLogEntry {
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
        detail: "elements=0".to_owned(),
        processing_micros: 1,
        prepare_micros: 2,
        current_micros: 3,
        select_micros: 4,
    });

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
