use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{
    Database,
    scheduler::{SchedulerStore, entities},
};
use tokio::sync::Mutex;

use super::{DATABASE_RUNNING_MESSAGE, execute_task_run, is_running};
use crate::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerResult, TaskConfigValue, TaskResult};

const TASK_CODE: &str = "request_record_cleanup";

#[tokio::test]
async fn advisory_lock_failure_records_skipped_without_running_task_body() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[advisory_lock_row(false)]])
        .append_query_results([[run_record("running", Some(DATABASE_RUNNING_MESSAGE))]])
        .append_query_results([[run_record("running", Some(DATABASE_RUNNING_MESSAGE))]])
        .append_query_results([[run_record("skipped_running", Some(DATABASE_RUNNING_MESSAGE))]])
        .append_query_results([[task_record(None)]])
        .append_query_results([[task_record(Some("skipped_running"))]])
        .into_connection();
    let database = Database::new(connection.clone());
    let running = Arc::new(Mutex::new(HashSet::new()));
    let ran = Arc::new(AtomicBool::new(false));

    execute_task_run(
        SchedulerStore::new(database.clone()),
        running.clone(),
        TASK_CODE.to_owned(),
        Arc::new(TestTask { ran: ran.clone() }),
        database,
        serde_json::json!({}),
    )
    .await;

    assert!(!ran.load(Ordering::SeqCst));
    assert!(!is_running(running, TASK_CODE).await);
    let statements = logged_statements(connection);
    let sql = sql_strings(&statements);
    let values = bound_string_values(&statements);
    assert!(sql.iter().any(|item| item.contains("pg_try_advisory_xact_lock")), "{sql:?}");
    assert!(sql.iter().any(|item| item == "ROLLBACK"), "{sql:?}");
    assert!(values.iter().any(|item| item == "skipped_running"), "{values:?}");
    assert!(values.iter().any(|item| item == DATABASE_RUNNING_MESSAGE), "{values:?}");
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
        last_started_at: None,
        last_finished_at: Some(now),
        last_status: last_status.map(str::to_owned),
        last_duration_ms: Some(0),
        last_error: None,
        created_at: now,
        updated_at: now,
    }
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
