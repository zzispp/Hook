use std::collections::HashMap;

use types::scheduler::ScheduledTask;

use super::{next_run_from_now, with_next_run_times};

const TASK_CODE: &str = "request_record_cleanup";

#[test]
fn with_next_run_times_formats_runtime_snapshot() {
    let task = scheduled_task(true);
    let next_at = time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(600);
    let next_runs = HashMap::from([(TASK_CODE.to_owned(), next_at)]);

    let tasks = with_next_run_times(vec![task], next_runs).unwrap();

    assert_eq!(tasks[0].next_run_at.as_deref(), Some("1970-01-01T00:10:00Z"));
}

#[test]
fn next_run_from_now_is_empty_for_disabled_task() {
    let task = scheduled_task(false);

    let next_run_at = next_run_from_now(&task).unwrap();

    assert_eq!(next_run_at, None);
}

fn scheduled_task(enabled: bool) -> ScheduledTask {
    ScheduledTask {
        code: TASK_CODE.into(),
        name_key: "name".into(),
        description_key: "description".into(),
        enabled,
        interval_seconds: 600,
        next_run_at: None,
        config: serde_json::json!({}),
        config_schema: Vec::new(),
        last_started_at: None,
        last_finished_at: None,
        last_status: None,
        last_duration_ms: None,
        last_error: None,
        created_at: "1970-01-01T00:00:00Z".into(),
        updated_at: "1970-01-01T00:00:00Z".into(),
    }
}
