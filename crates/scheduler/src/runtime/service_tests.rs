use super::{next_attempt_at, next_attempt_delay};

const TASK_CODE: &str = "request_record_cleanup";

#[test]
fn next_attempt_at_uses_next_run_when_task_is_unlocked() {
    let record = task_record(ts(600), None);

    let attempt_at = next_attempt_at(&record);

    assert_eq!(attempt_at, ts(600));
}

#[test]
fn next_attempt_at_waits_for_active_claim_lease() {
    let record = task_record(ts(600), Some(ts(900)));

    let attempt_at = next_attempt_at(&record);

    assert_eq!(attempt_at, ts(900));
}

#[test]
fn next_attempt_delay_is_zero_when_task_is_due() {
    let record = task_record(ts(600), None);

    let delay = next_attempt_delay(&record, ts(601)).unwrap();

    assert_eq!(delay, std::time::Duration::ZERO);
}

fn task_record(next_run_at: time::OffsetDateTime, locked_until: Option<time::OffsetDateTime>) -> storage::scheduler::entities::scheduled_tasks::Model {
    let now = time::OffsetDateTime::UNIX_EPOCH;
    storage::scheduler::entities::scheduled_tasks::Model {
        code: TASK_CODE.into(),
        enabled: true,
        interval_seconds: 600,
        config: "{}".into(),
        next_run_at,
        locked_until,
        locked_by: None,
        last_started_at: None,
        last_finished_at: None,
        last_status: None,
        last_duration_ms: None,
        last_error: None,
        created_at: now,
        updated_at: now,
    }
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds)
}
