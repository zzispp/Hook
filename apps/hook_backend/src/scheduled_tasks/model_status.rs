use std::sync::Arc;

use ::model_status::application::ModelStatusDispatchOptions;
use scheduler::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerResult, TaskConfigValue, TaskResult};
use storage::{model_status::ModelStatusStore, scheduler::task_definition};
use types::scheduler::ScheduledTaskDefinition;

use super::{integer_config, integer_fields, model_status_error, storage_error, validate_positive_integer};

const DISPATCH_INTERVAL_SECONDS: i64 = 60;
const DISPATCH_BATCH_SIZE: i64 = 20;
const DISPATCH_CONCURRENCY: i64 = 4;
const PROVIDER_KEY_MIN_INTERVAL_SECONDS: i64 = 1;
const RUNS_CLEANUP_INTERVAL_SECONDS: i64 = 300;
const RUNS_CLEANUP_RETENTION_DAYS: i64 = 90;

#[derive(Clone)]
pub(super) struct ModelStatusCheckDispatchTask {
    pub(super) model_status_service: Arc<dyn ::model_status::application::ModelStatusUseCase>,
}

#[derive(Clone, Copy)]
pub(super) struct ModelStatusRunsCleanupTask;

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for ModelStatusCheckDispatchTask {
    fn definition(&self) -> ScheduledTaskDefinition {
        task_definition(
            "model_status_check_dispatch",
            "scheduledTasks.definitions.modelStatusCheckDispatch.name",
            "scheduledTasks.definitions.modelStatusCheckDispatch.description",
            DISPATCH_INTERVAL_SECONDS,
            serde_json::json!({
                "batch_size": DISPATCH_BATCH_SIZE,
                "concurrency": DISPATCH_CONCURRENCY,
                "provider_key_min_interval_seconds": PROVIDER_KEY_MIN_INTERVAL_SECONDS
            }),
            integer_fields(&[
                ("batch_size", "scheduledTasks.config.modelStatusCheckDispatch.batchSize", 1),
                ("concurrency", "scheduledTasks.config.modelStatusCheckDispatch.concurrency", 1),
                (
                    "provider_key_min_interval_seconds",
                    "scheduledTasks.config.modelStatusCheckDispatch.providerKeyMinIntervalSeconds",
                    1,
                ),
            ]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "batch_size", 1)?;
        validate_positive_integer(config, "concurrency", 1)?;
        validate_positive_integer(config, "provider_key_min_interval_seconds", 1)
    }

    async fn run(&self, _ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let batch_size = positive_u64_config(&config, "batch_size")?;
        let concurrency = positive_usize_config(&config, "concurrency")?;
        let provider_key_min_interval_seconds = integer_config(&config, "provider_key_min_interval_seconds")?;
        let report = self
            .model_status_service
            .run_due_checks(ModelStatusDispatchOptions {
                limit: batch_size,
                concurrency,
                provider_key_min_interval_seconds,
            })
            .await
            .map_err(model_status_error)?;
        Ok(Some(format!("probed_count={}, deferred_count={}", report.probed_count, report.deferred_count)))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for ModelStatusRunsCleanupTask {
    fn definition(&self) -> ScheduledTaskDefinition {
        task_definition(
            "model_status_runs_cleanup",
            "scheduledTasks.definitions.modelStatusRunsCleanup.name",
            "scheduledTasks.definitions.modelStatusRunsCleanup.description",
            RUNS_CLEANUP_INTERVAL_SECONDS,
            serde_json::json!({
                "retention_days": RUNS_CLEANUP_RETENTION_DAYS
            }),
            integer_fields(&[("retention_days", "scheduledTasks.config.modelStatusRunsCleanup.retentionDays", 1)]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "retention_days", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let retention_days = integer_config(&config, "retention_days")?;
        let cutoff = time::OffsetDateTime::now_utc() - time::Duration::days(retention_days);
        let report = ModelStatusStore::new(ctx.database).delete_history_before(cutoff).await.map_err(storage_error)?;
        Ok(Some(format!(
            "deleted_runs={}, deleted_hourly_stats={}",
            report.deleted_runs, report.deleted_hourly_stats
        )))
    }
}

fn positive_u64_config(config: &TaskConfigValue, key: &str) -> SchedulerResult<u64> {
    u64::try_from(integer_config(config, key)?).map_err(|_| SchedulerError::InvalidInput(format!("{key} must be greater than 0")))
}

fn positive_usize_config(config: &TaskConfigValue, key: &str) -> SchedulerResult<usize> {
    usize::try_from(integer_config(config, key)?).map_err(|_| SchedulerError::InvalidInput(format!("{key} must be greater than 0")))
}
