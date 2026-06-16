use std::sync::Arc;

use storage::{StorageResult, scheduler::SchedulerStore};
use types::{
    pagination::{Page, PageSliceRequest},
    scheduler::{ScheduledTask, ScheduledTaskDefinition, ScheduledTaskRun, ScheduledTaskRunListRequest},
};

use crate::runtime::{SchedulerError, SchedulerRegistry, SchedulerResult};

const MIN_PAGE_NUMBER: u64 = 1;
const MIN_PAGE_SIZE: u64 = 1;
const MAX_PAGE_SIZE: u64 = 100;

pub async fn list_tasks(store: &SchedulerStore, registry: &Arc<SchedulerRegistry>) -> SchedulerResult<Vec<ScheduledTask>> {
    let definitions = registry.definitions();
    store.ensure_registered_tasks(&definitions).await?;
    store.list_tasks(&definitions).await.map_err(Into::into)
}

pub async fn list_runs(store: &SchedulerStore, request: ScheduledTaskRunListRequest) -> SchedulerResult<Page<ScheduledTaskRun>> {
    let slice = slice_request(request.page, request.page_size)?;
    store
        .page_runs(slice, request.task_code.as_deref(), request.status.as_deref())
        .await
        .map_err(Into::into)
}

pub fn task_definition(registry: &Arc<SchedulerRegistry>, code: &str) -> SchedulerResult<ScheduledTaskDefinition> {
    registry
        .definitions()
        .into_iter()
        .find(|definition| definition.code == code)
        .ok_or_else(|| SchedulerError::NotFound(code.to_owned()))
}

pub fn slice_request(page: u64, page_size: u64) -> SchedulerResult<PageSliceRequest> {
    if page < MIN_PAGE_NUMBER {
        return Err(SchedulerError::InvalidInput("page must be greater than 0".into()));
    }
    if !(MIN_PAGE_SIZE..=MAX_PAGE_SIZE).contains(&page_size) {
        return Err(SchedulerError::InvalidInput(format!(
            "page_size must be between {MIN_PAGE_SIZE} and {MAX_PAGE_SIZE}"
        )));
    }
    Ok(PageSliceRequest {
        offset: (page - MIN_PAGE_NUMBER) * page_size,
        limit: page_size,
        page,
        page_size,
    })
}

pub fn next_runtime_config(current: &storage::scheduler::ScheduledTaskRecord, update_config: Option<serde_json::Value>) -> StorageResult<serde_json::Value> {
    match update_config {
        Some(value) => Ok(value),
        None => current.runtime_config(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use storage::{Database, scheduler::SchedulerStore};
    use types::scheduler::ScheduledTaskDefinition;

    use crate::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerRegistry, SchedulerResult, TaskConfigValue, TaskResult, query::list_tasks};

    #[tokio::test]
    async fn list_tasks_registers_missing_registry_tasks() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<storage::scheduler::entities::scheduled_tasks::Model>::new()])
            .append_query_results([[task_record("recharge_payment_poll")]])
            .append_query_results([vec![task_record("recharge_payment_poll")]])
            .into_connection();
        let store = SchedulerStore::new(Database::new(connection.clone()));
        let registry = Arc::new(test_registry());

        let tasks = list_tasks(&store, &registry).await.unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].code, "recharge_payment_poll");
        let statements = logged_sql(connection);
        assert!(statements.iter().any(|sql| sql.contains("INSERT INTO \"scheduled_tasks\"")), "{statements:?}");
    }

    #[tokio::test]
    async fn list_tasks_ignores_unregistered_database_rows() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[task_record("recharge_payment_poll")]])
            .append_query_results([[task_record("request_record_stale_sweep")]])
            .into_connection();
        let store = SchedulerStore::new(Database::new(connection));
        let registry = Arc::new(test_registry());

        let tasks = list_tasks(&store, &registry).await.unwrap();

        assert!(tasks.is_empty());
    }

    fn test_registry() -> SchedulerRegistry {
        let mut registry = SchedulerRegistry::new();
        registry.register(TestTask).unwrap();
        registry
    }

    #[derive(Clone, Copy)]
    struct TestTask;

    #[async_trait]
    impl ScheduledTaskLifecycle for TestTask {
        fn definition(&self) -> ScheduledTaskDefinition {
            storage::scheduler::task_definition(
                "recharge_payment_poll",
                "scheduledTasks.definitions.rechargePaymentPoll.name",
                "scheduledTasks.definitions.rechargePaymentPoll.description",
                60,
                serde_json::json!({"limit": 50}),
                Vec::new(),
            )
        }

        fn validate_config(&self, _config: &TaskConfigValue) -> SchedulerResult<()> {
            Ok(())
        }

        async fn run(&self, _ctx: ScheduleTaskContext, _config: TaskConfigValue) -> TaskResult {
            Ok(None)
        }
    }

    fn task_record(code: &str) -> storage::scheduler::entities::scheduled_tasks::Model {
        let now = time::OffsetDateTime::UNIX_EPOCH;
        storage::scheduler::entities::scheduled_tasks::Model {
            code: code.into(),
            enabled: true,
            interval_seconds: 60,
            config: serde_json::json!({"limit": 50}).to_string(),
            next_run_at: now + time::Duration::seconds(60),
            locked_until: None,
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

    fn logged_sql(connection: sea_orm::DatabaseConnection) -> Vec<String> {
        connection
            .into_transaction_log()
            .iter()
            .flat_map(|entry| entry.statements())
            .map(|statement| statement.sql.clone())
            .collect()
    }
}
