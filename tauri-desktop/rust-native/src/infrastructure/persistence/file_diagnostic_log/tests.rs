use super::*;
use crate::application::input_source_switcher::SwitchDecision;

#[test]
fn writes_jsonl_switch_records() {
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
    });

    let body = fs::read_to_string(sink.path()).expect("read");
    assert!(body.contains("\"kind\":\"session_start\""));
    assert!(body.contains("\"kind\":\"switch\""));
    assert!(body.contains("\"decision\":\"switched\""));
    assert!(body.contains("postSelect=match"));
    assert!(body.contains("\"processingMicros\":1"));

    let _ = fs::remove_dir_all(&dir);
}
