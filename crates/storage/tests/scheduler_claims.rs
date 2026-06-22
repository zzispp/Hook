use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use storage::{
    Database,
    scheduler::{ScheduledTaskRunRecordPatch, ScheduledTaskRunStatus, SchedulerStore, entities},
};

const TASK_CODE: &str = "request_record_cleanup";
const LOCK_OWNER: &str = "claim-owner";

#[tokio::test]
async fn claim_due_task_uses_atomic_due_window_claim() {
    let now = ts(600);
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[task_record(now)]])
        .into_connection();
    let store = SchedulerStore::new(Database::new(connection.clone()));

    let claim = store.claim_due_task(TASK_CODE, now).await.unwrap().unwrap();

    assert_eq!(claim.record.code, TASK_CODE);
    assert_eq!(claim.started_at, now);
    assert!(!claim.lock_owner.is_empty());
    let statements = logged_statements(connection);
    let sql = sql_strings(&statements);
    let values = bound_string_values(&statements);
    assert!(sql.iter().any(|item| item.contains("FOR UPDATE SKIP LOCKED")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("next_run_at <= $2")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("(locked_until IS NULL OR locked_until <= $2)")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("lease_seconds * INTERVAL '1 second'")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("locked_by = $3")), "{sql:?}");
    assert!(values.iter().any(|item| item == TASK_CODE), "{values:?}");
    assert!(values.iter().any(|item| item == &claim.lock_owner), "{values:?}");
}

#[tokio::test]
async fn finish_claimed_task_run_requires_matching_lock_owner() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .into_connection();
    let store = SchedulerStore::new(Database::new(connection.clone()));

    let updated = store.finish_claimed_task_run(TASK_CODE, LOCK_OWNER, run_patch()).await.unwrap();

    assert!(!updated);
    let statements = logged_statements(connection);
    let sql = sql_strings(&statements);
    let values = bound_string_values(&statements);
    assert!(sql.iter().any(|item| item.contains("UPDATE \"scheduled_tasks\"")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("\"locked_by\" =")), "{sql:?}");
    assert!(values.iter().any(|item| item == LOCK_OWNER), "{values:?}");
    assert!(values.iter().any(|item| item == "failed"), "{values:?}");
}

fn task_record(now: time::OffsetDateTime) -> entities::scheduled_tasks::Model {
    entities::scheduled_tasks::Model {
        code: TASK_CODE.into(),
        enabled: true,
        interval_seconds: 60,
        lease_seconds: 180,
        config: "{}".into(),
        next_run_at: now + time::Duration::seconds(60),
        locked_until: Some(now + time::Duration::seconds(60)),
        locked_by: Some(LOCK_OWNER.into()),
        last_started_at: Some(now),
        last_finished_at: None,
        last_status: None,
        last_duration_ms: None,
        last_error: None,
        created_at: now,
        updated_at: now,
    }
}

fn run_patch() -> ScheduledTaskRunRecordPatch {
    ScheduledTaskRunRecordPatch {
        status: ScheduledTaskRunStatus::Failed,
        finished_at: ts(660),
        duration_ms: 60_000,
        message: None,
        error: Some("failed".into()),
    }
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds)
}

fn logged_statements(connection: sea_orm::DatabaseConnection) -> Vec<sea_orm::Statement> {
    connection.into_transaction_log().iter().flat_map(|entry| entry.statements()).cloned().collect()
}

fn sql_strings(statements: &[sea_orm::Statement]) -> Vec<String> {
    statements.iter().map(|statement| statement.sql.clone()).collect()
}

fn bound_string_values(statements: &[sea_orm::Statement]) -> Vec<String> {
    statements
        .iter()
        .filter_map(|statement| statement.values.as_ref())
        .flat_map(|values| values.0.iter())
        .filter_map(|value| match value {
            Value::String(Some(text)) => Some(text.clone()),
            _ => None,
        })
        .collect()
}
