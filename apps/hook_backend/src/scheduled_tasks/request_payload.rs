use scheduler::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerResult, TaskConfigValue, TaskResult};
use storage::provider::{ProviderStore, RequestPayloadBackfillOptions, RequestPayloadBackfillResult, RequestPayloadStaleSweepResult};

use super::{integer_config, integer_fields, storage_error, unsigned_config, validate_positive_integer};

const PAYLOAD_BACKFILL_INTERVAL_SECONDS: i64 = 600;
const PAYLOAD_STALE_SWEEP_INTERVAL_SECONDS: i64 = 300;
const DEFAULT_PAYLOAD_BACKFILL_BATCH_SIZE: i64 = 100;
const DEFAULT_PAYLOAD_STALE_PENDING_MINUTES: i64 = 15;
const DEFAULT_PAYLOAD_RETENTION_DAYS: i64 = 1;

#[derive(Clone, Copy)]
pub(super) struct RequestPayloadBackfillTask;

#[derive(Clone, Copy)]
pub(super) struct RequestPayloadStaleSweepTask;

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RequestPayloadBackfillTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        storage::scheduler::task_definition(
            "request_payload_backfill",
            "scheduledTasks.definitions.requestPayloadBackfill.name",
            "scheduledTasks.definitions.requestPayloadBackfill.description",
            PAYLOAD_BACKFILL_INTERVAL_SECONDS,
            serde_json::json!({ "batch_size": DEFAULT_PAYLOAD_BACKFILL_BATCH_SIZE }),
            integer_fields(&[("batch_size", "scheduledTasks.config.requestPayloadBackfill.batchSize", 1)]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "batch_size", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let now = time::OffsetDateTime::now_utc();
        let result = ProviderStore::new(ctx.database)
            .backfill_legacy_request_payloads(RequestPayloadBackfillOptions {
                batch_size: unsigned_config(&config, "batch_size")?,
                minimum_created_at: now - time::Duration::days(DEFAULT_PAYLOAD_RETENTION_DAYS),
            })
            .await
            .map_err(storage_error)?;
        Ok(Some(backfill_message(result)))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RequestPayloadStaleSweepTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        storage::scheduler::task_definition(
            "request_payload_stale_sweep",
            "scheduledTasks.definitions.requestPayloadStaleSweep.name",
            "scheduledTasks.definitions.requestPayloadStaleSweep.description",
            PAYLOAD_STALE_SWEEP_INTERVAL_SECONDS,
            serde_json::json!({ "pending_timeout_minutes": DEFAULT_PAYLOAD_STALE_PENDING_MINUTES }),
            integer_fields(&[(
                "pending_timeout_minutes",
                "scheduledTasks.config.requestPayloadStaleSweep.pendingTimeoutMinutes",
                1,
            )]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "pending_timeout_minutes", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let timeout = integer_config(&config, "pending_timeout_minutes")?;
        let result = ProviderStore::new(ctx.database)
            .mark_stale_request_payloads_failed(time::OffsetDateTime::now_utc() - time::Duration::minutes(timeout))
            .await
            .map_err(storage_error)?;
        Ok(Some(payload_stale_message(result)))
    }
}

fn backfill_message(result: RequestPayloadBackfillResult) -> String {
    format!(
        "records_backfilled={}, candidates_backfilled={}",
        result.records_backfilled, result.candidates_backfilled
    )
}

fn payload_stale_message(result: RequestPayloadStaleSweepResult) -> String {
    format!("failed_payloads={}", result.failed_payloads)
}
