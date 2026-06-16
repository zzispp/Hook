use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use storage::{
    Database,
    scheduler::{ScheduledTaskClaim, SchedulerStore, entities},
};
use tokio::sync::Mutex;

use super::{DATABASE_RUNNING_MESSAGE, LOCAL_RUNNING_MESSAGE, dispatch_task, execute_task_run, is_running};
use crate::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerResult, TaskConfigValue, TaskResult};

const TASK_CODE: &str = "request_record_cleanup";

#[tokio::test]
async fn advisory_lock_failure_records_skipped_without_running_task_body() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[advisory_lock_row(false)]])
        .append_query_results([[run_record("running", Some(DATABASE_RUNNING_MESSAGE))]])
        .append_query_results([[run_record("running", Some(DATABASE_RUNNING_MESSAGE))]])
        .append_query_results([[run_record("skipped_running", Some(DATABASE_RUNNING_MESSAGE))]])
        .append_exec_results([updated_row()])
        .into_connection();
    let database = Database::new(connection.clone());
    let running = Arc::new(Mutex::new(HashSet::new()));
    let ran = Arc::new(AtomicBool::new(false));

    execute_task_run(
        SchedulerStore::new(database.clone()),
        running.clone(),
        Arc::new(TestTask { ran: ran.clone() }),
        database,
        task_claim(),
    )
    .await;

    assert!(!ran.load(Ordering::SeqCst));
    assert!(!is_running(running, TASK_CODE).await);
    let statements = logged_statements(connection);
    let sql = sql_strings(&statements);
    let values = bound_string_values(&statements);
    assert!(sql.iter().any(|item| item.contains("pg_try_advisory_xact_lock")), "{sql:?}");
    assert!(sql.iter().any(|item| item == "ROLLBACK"), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("UPDATE \"scheduled_tasks\"")), "{sql:?}");
    assert!(values.iter().any(|item| item == "skipped_running"), "{values:?}");
    assert!(values.iter().any(|item| item == DATABASE_RUNNING_MESSAGE), "{values:?}");
}

#[tokio::test]
async fn dispatch_without_due_claim_does_not_create_task_run() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<entities::scheduled_tasks::Model>::new()])
        .into_connection();
    let database = Database::new(connection.clone());
    let running = Arc::new(Mutex::new(HashSet::new()));
    let ran = Arc::new(AtomicBool::new(false));

    dispatch_task(
        &SchedulerStore::new(database.clone()),
        running,
        TASK_CODE,
        Arc::new(TestTask { ran: ran.clone() }),
        database,
    )
    .await
    .unwrap();

    assert!(!ran.load(Ordering::SeqCst));
    let sql = sql_strings(&logged_statements(connection));
    assert!(sql.iter().any(|item| item.contains("FOR UPDATE SKIP LOCKED")), "{sql:?}");
    assert!(sql.iter().all(|item| !item.contains("\"scheduled_task_runs\"")), "{sql:?}");
}

#[tokio::test]
async fn dispatch_records_local_running_skip_only_after_due_claim() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[task_record(None)]])
        .append_query_results([[run_record("running", Some(LOCAL_RUNNING_MESSAGE))]])
        .append_query_results([[run_record("running", Some(LOCAL_RUNNING_MESSAGE))]])
        .append_query_results([[run_record("skipped_running", Some(LOCAL_RUNNING_MESSAGE))]])
        .append_exec_results([updated_row()])
        .into_connection();
    let database = Database::new(connection.clone());
    let running = Arc::new(Mutex::new(HashSet::from([TASK_CODE.to_owned()])));
    let ran = Arc::new(AtomicBool::new(false));

    dispatch_task(
        &SchedulerStore::new(database.clone()),
        running.clone(),
        TASK_CODE,
        Arc::new(TestTask { ran: ran.clone() }),
        database,
    )
    .await
    .unwrap();

    assert!(!ran.load(Ordering::SeqCst));
    assert!(is_running(running, TASK_CODE).await);
    let statements = logged_statements(connection);
    let sql = sql_strings(&statements);
    let values = bound_string_values(&statements);
    assert!(statement_position(&sql, "FOR UPDATE SKIP LOCKED") < statement_position(&sql, "INSERT INTO \"scheduled_task_runs\""));
    assert!(sql.iter().all(|item| !item.contains("pg_try_advisory_xact_lock")), "{sql:?}");
    assert!(values.iter().any(|item| item == "skipped_running"), "{values:?}");
    assert!(values.iter().any(|item| item == LOCAL_RUNNING_MESSAGE), "{values:?}");
}

struct TestTask {
    ran: Arc<AtomicBool>,
}

#[async_trait]
impl ScheduledTaskLifecycle for TestTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        storage::scheduler::task_definition(TASK_CODE, "name", "description", 60, serde_json::json!({}), Vec::new())
    }

    fn validate_config(&self, _config: &TaskConfigValue) -> SchedulerResult<()> {
        Ok(())
    }

    async fn run(&self, _ctx: ScheduleTaskContext, _config: TaskConfigValue) -> TaskResult {
        self.ran.store(true, Ordering::SeqCst);
        Ok(None)
    }
}

fn advisory_lock_row(acquired: bool) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("acquired", Value::from(acquired))])
}

fn run_record(status: &str, message: Option<&str>) -> entities::scheduled_task_runs::Model {
    let now = time::OffsetDateTime::UNIX_EPOCH;
    entities::scheduled_task_runs::Model {
        id: "run-1".into(),
        task_code: TASK_CODE.into(),
        status: status.into(),
        started_at: now,
        finished_at: Some(now),
        duration_ms: Some(0),
        message: message.map(str::to_owned),
        error: None,
    }
}

fn task_record(last_status: Option<&str>) -> entities::scheduled_tasks::Model {
    let now = time::OffsetDateTime::UNIX_EPOCH;
    entities::scheduled_tasks::Model {
        code: TASK_CODE.into(),
        enabled: true,
        interval_seconds: 60,
        config: "{}".into(),
        next_run_at: now + time::Duration::seconds(60),
        locked_until: Some(now + time::Duration::seconds(60)),
        locked_by: Some("claim-owner".into()),
        last_started_at: None,
        last_finished_at: Some(now),
        last_status: last_status.map(str::to_owned),
        last_duration_ms: Some(0),
        last_error: None,
        created_at: now,
        updated_at: now,
    }
}

fn task_claim() -> ScheduledTaskClaim {
    ScheduledTaskClaim {
        record: task_record(None),
        lock_owner: "claim-owner".into(),
        started_at: time::OffsetDateTime::UNIX_EPOCH,
    }
}

fn updated_row() -> MockExecResult {
    MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
    }
}

fn logged_statements(connection: sea_orm::DatabaseConnection) -> Vec<sea_orm::Statement> {
    connection.into_transaction_log().iter().flat_map(|entry| entry.statements()).cloned().collect()
}

fn sql_strings(statements: &[sea_orm::Statement]) -> Vec<String> {
    statements.iter().map(|statement| statement.sql.clone()).collect()
}

fn statement_position(sql: &[String], pattern: &str) -> usize {
    sql.iter().position(|item| item.contains(pattern)).unwrap()
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
