use super::{AppUpdateSession, UpdatePhase, UpdateSchedule, FAILURE_RETRY_MS, SUCCESS_INTERVAL_MS};

fn session() -> AppUpdateSession {
    AppUpdateSession::new(UpdateSchedule::defaults(), true)
}

#[test]
fn first_schedule_is_due_until_a_successful_check() {
    let schedule = UpdateSchedule::defaults();
    assert!(schedule.is_due(0, false));
    let mut checked = schedule;
    checked.record_success(1_000);
    assert!(!checked.is_due(1_000 + SUCCESS_INTERVAL_MS - 1, false));
    assert!(checked.is_due(1_000 + SUCCESS_INTERVAL_MS, false));
    assert_eq!(checked.last_checked_at_unix_ms, Some(1_000));
}

#[test]
fn failed_check_keeps_last_success_and_retries_in_one_hour() {
    let mut schedule = UpdateSchedule::defaults();
    schedule.record_success(5_000);
    schedule.record_failure(9_000);
    assert_eq!(schedule.last_checked_at_unix_ms, Some(5_000));
    assert_eq!(
        schedule.next_check_at_unix_ms,
        Some(9_000 + FAILURE_RETRY_MS)
    );
    assert!(!schedule.is_due(9_000 + FAILURE_RETRY_MS - 1, false));
    assert!(schedule.is_due(9_000 + FAILURE_RETRY_MS, false));
}

#[test]
fn automatic_check_stays_off_when_disabled() {
    let mut schedule = UpdateSchedule::defaults();
    schedule.auto_check = false;
    assert!(!schedule.is_due(0, false));
    assert!(!schedule.is_due(u64::MAX, true));
}

#[test]
fn legacy_last_check_defers_the_next_automatic_check() {
    let mut schedule = UpdateSchedule::defaults();
    schedule.adopt_legacy(Some(false), Some(false), Some(10_000));
    assert!(!schedule.auto_check);
    assert!(!schedule.auto_download);
    assert_eq!(schedule.last_checked_at_unix_ms, Some(10_000));
    assert_eq!(
        schedule.next_check_at_unix_ms,
        Some(10_000 + SUCCESS_INTERVAL_MS)
    );
}

#[test]
fn pending_version_is_redownloaded_once_per_process() {
    let mut schedule = UpdateSchedule::defaults();
    schedule.record_success(0);
    schedule.pending_version = Some("0.2.0".to_owned());
    let mut session = AppUpdateSession::new(schedule, true);
    assert!(session.scheduled_check_due(1));
    session.begin_check();
    assert!(!session.scheduled_check_due(1));
    session.note_available(1, "0.2.0".to_owned());
    assert_eq!(session.schedule.pending_version.as_deref(), Some("0.2.0"));
    assert!(!session.scheduled_check_due(1));
    assert!(session.install_ready());
    assert!(!session.tray_install_ready());
    session.begin_download();
    session.note_downloaded();
    assert!(session.tray_install_ready());
    assert!(matches!(session.phase, UpdatePhase::Downloaded { .. }));
}

#[test]
fn up_to_date_clears_a_pending_version() {
    let mut session = session();
    session.note_available(1, "0.2.0".to_owned());
    session.note_up_to_date(2);
    assert!(session.schedule.pending_version.is_none());
    assert!(!session.install_ready());
}

#[test]
fn check_failure_does_not_move_the_displayed_check_time() {
    let mut session = session();
    session.note_up_to_date(4_000);
    session.note_check_failed(8_000);
    assert_eq!(session.schedule.last_checked_at_unix_ms, Some(4_000));
    assert!(matches!(session.phase, UpdatePhase::CheckFailed));
}

#[test]
fn turning_auto_download_on_only_starts_a_pending_download() {
    let mut session = session();
    assert!(!session.set_auto_download(true));
    session.note_available(1, "0.2.0".to_owned());
    session.set_auto_download(false);
    assert!(session.set_auto_download(true));
    session.begin_download();
    session.note_downloaded();
    assert!(!session.set_auto_download(true));
}

#[test]
fn accept_client_runs_once() {
    let mut session = AppUpdateSession::new(UpdateSchedule::defaults(), false);
    let first = session.accept_client(Some(false), None, Some(3_000));
    assert!(first.persist);
    assert!(first.clear_legacy);
    let second = session.accept_client(Some(true), Some(true), None);
    assert!(!second.persist);
    assert!(second.clear_legacy);
    assert!(!session.schedule.auto_check);
    assert_eq!(session.schedule.last_checked_at_unix_ms, Some(3_000));
}

#[test]
fn late_client_ack_keeps_an_off_switch_without_rewinding_a_newer_check() {
    let mut session = AppUpdateSession::new(UpdateSchedule::defaults(), false);
    assert!(session.release_without_client());
    session.note_up_to_date(50_000);
    let ack = session.accept_client(Some(false), Some(true), Some(1_000));
    assert!(ack.persist);
    assert!(ack.clear_legacy);
    assert!(!session.schedule.auto_check);
    assert!(session.schedule.auto_download);
    assert_eq!(session.schedule.last_checked_at_unix_ms, Some(50_000));
}

#[test]
fn late_client_ack_uses_the_legacy_check_when_none_has_succeeded() {
    let mut session = AppUpdateSession::new(UpdateSchedule::defaults(), false);
    assert!(session.release_without_client());
    session.accept_client(Some(false), None, Some(3_000));
    assert!(!session.schedule.auto_check);
    assert_eq!(session.schedule.last_checked_at_unix_ms, Some(3_000));
    assert!(!session.scheduled_check_due(3_000));
}

#[test]
fn stored_client_ack_ignores_a_later_legacy_payload() {
    let mut schedule = UpdateSchedule::defaults();
    schedule.client_acked = true;
    let mut session = AppUpdateSession::new(schedule, true);
    let ack = session.accept_client(Some(false), None, Some(1));
    assert!(!ack.persist);
    assert!(ack.clear_legacy);
    assert!(session.schedule.auto_check);
}

#[test]
fn explicit_auto_check_blocks_a_later_legacy_payload() {
    let mut session = AppUpdateSession::new(UpdateSchedule::defaults(), false);
    session.set_auto_check(false);
    let ack = session.accept_client(Some(true), None, None);
    assert!(!ack.persist);
    assert!(!session.schedule.auto_check);
}
