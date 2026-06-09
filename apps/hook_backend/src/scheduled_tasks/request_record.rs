use std::time::Duration;

use scheduler::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerResult, TaskConfigValue, TaskResult};
use storage::provider::{
    ProviderStore, RequestPartitionMaintenanceOptions, RequestPartitionMaintenanceResult, RequestRecordCleanupOptions, RequestRecordCleanupResult,
};

use super::{integer_config, integer_fields, storage_error, unsigned_config, validate_positive_integer};

const STALE_SWEEP_INTERVAL_SECONDS: i64 = 300;
const CLEANUP_INTERVAL_SECONDS: i64 = 600;
const PARTITION_MAINTENANCE_INTERVAL_SECONDS: i64 = 3600;
const STALE_PENDING_TIMEOUT_MINUTES: i64 = 10;
const STALE_STREAMING_TIMEOUT_MINUTES: i64 = 10;
const DEFAULT_RECORD_RETENTION_DAYS: i64 = 3;
const DEFAULT_PAYLOAD_RETENTION_DAYS: i64 = 1;
const DEFAULT_PARTITION_FUTURE_DAYS: i64 = 3;
const DEFAULT_DELETE_BATCH_SIZE: i64 = 200;
const DEFAULT_COMPRESS_BATCH_SIZE: i64 = 50;
const DEFAULT_MAX_RUNTIME_SECONDS: i64 = 120;
const DEFAULT_BATCH_SLEEP_MS: i64 = 100;
const DEFAULT_STATEMENT_TIMEOUT_SECONDS: i64 = 15;
const DEFAULT_LOCK_TIMEOUT_SECONDS: i64 = 2;

#[derive(Clone, Copy)]
pub(super) struct RequestRecordCleanupTask;

#[derive(Clone, Copy)]
pub(super) struct RequestRecordStaleSweepTask;

#[derive(Clone, Copy)]
pub(super) struct RequestRecordPartitionMaintenanceTask;

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RequestRecordCleanupTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        storage::scheduler::task_definition(
            "request_record_cleanup",
            "scheduledTasks.definitions.requestRecordCleanup.name",
            "scheduledTasks.definitions.requestRecordCleanup.description",
            CLEANUP_INTERVAL_SECONDS,
            default_cleanup_config(),
            cleanup_config_fields(),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_cleanup_config(config)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let result = ProviderStore::new(ctx.database)
            .cleanup_request_records(cleanup_options(time::OffsetDateTime::now_utc(), &config)?)
            .await
            .map_err(storage_error)?;
        Ok(Some(cleanup_message(result)))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RequestRecordStaleSweepTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        storage::scheduler::task_definition(
            "request_record_stale_sweep",
            "scheduledTasks.definitions.requestRecordStaleSweep.name",
            "scheduledTasks.definitions.requestRecordStaleSweep.description",
            STALE_SWEEP_INTERVAL_SECONDS,
            serde_json::json!({
                "pending_timeout_minutes": STALE_PENDING_TIMEOUT_MINUTES,
                "streaming_timeout_minutes": STALE_STREAMING_TIMEOUT_MINUTES
            }),
            integer_fields(&[
                (
                    "pending_timeout_minutes",
                    "scheduledTasks.config.requestRecordStaleSweep.pendingTimeoutMinutes",
                    1,
                ),
                (
                    "streaming_timeout_minutes",
                    "scheduledTasks.config.requestRecordStaleSweep.streamingTimeoutMinutes",
                    1,
                ),
            ]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "pending_timeout_minutes", 1)?;
        validate_positive_integer(config, "streaming_timeout_minutes", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let now = time::OffsetDateTime::now_utc();
        let pending = integer_config(&config, "pending_timeout_minutes")?;
        let streaming = integer_config(&config, "streaming_timeout_minutes")?;
        let result = ProviderStore::new(ctx.database)
            .mark_stale_request_records_failed(now - time::Duration::minutes(pending), now - time::Duration::minutes(streaming))
            .await
            .map_err(storage_error)?;
        Ok(Some(format!(
            "stale_request_records={}, stale_request_candidates={}",
            result.request_records, result.request_candidates
        )))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RequestRecordPartitionMaintenanceTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        storage::scheduler::task_definition(
            "request_record_partition_maintenance",
            "scheduledTasks.definitions.requestRecordPartitionMaintenance.name",
            "scheduledTasks.definitions.requestRecordPartitionMaintenance.description",
            PARTITION_MAINTENANCE_INTERVAL_SECONDS,
            serde_json::json!({
                "record_retention_days": DEFAULT_RECORD_RETENTION_DAYS,
                "payload_retention_days": DEFAULT_PAYLOAD_RETENTION_DAYS,
                "future_days": DEFAULT_PARTITION_FUTURE_DAYS
            }),
            integer_fields(&[
                (
                    "record_retention_days",
                    "scheduledTasks.config.requestRecordPartitionMaintenance.recordRetentionDays",
                    1,
                ),
                (
                    "payload_retention_days",
                    "scheduledTasks.config.requestRecordPartitionMaintenance.payloadRetentionDays",
                    1,
                ),
                ("future_days", "scheduledTasks.config.requestRecordPartitionMaintenance.futureDays", 0),
            ]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "record_retention_days", 1)?;
        validate_positive_integer(config, "payload_retention_days", 1)?;
        validate_positive_integer(config, "future_days", 0)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let result = ProviderStore::new(ctx.database)
            .maintain_request_record_partitions(partition_options(time::OffsetDateTime::now_utc(), &config)?)
            .await
            .map_err(storage_error)?;
        Ok(Some(partition_message(result)))
    }
}

fn default_cleanup_config() -> serde_json::Value {
    serde_json::json!({
        "record_retention_days": DEFAULT_RECORD_RETENTION_DAYS,
        "payload_retention_days": DEFAULT_PAYLOAD_RETENTION_DAYS,
        "delete_batch_size": DEFAULT_DELETE_BATCH_SIZE,
        "compress_batch_size": DEFAULT_COMPRESS_BATCH_SIZE,
        "max_runtime_seconds": DEFAULT_MAX_RUNTIME_SECONDS,
        "batch_sleep_ms": DEFAULT_BATCH_SLEEP_MS,
        "statement_timeout_seconds": DEFAULT_STATEMENT_TIMEOUT_SECONDS,
        "lock_timeout_seconds": DEFAULT_LOCK_TIMEOUT_SECONDS
    })
}

fn cleanup_config_fields() -> Vec<types::scheduler::ScheduledTaskConfigField> {
    integer_fields(&[
        ("record_retention_days", "scheduledTasks.config.requestRecordCleanup.recordRetentionDays", 1),
        ("payload_retention_days", "scheduledTasks.config.requestRecordCleanup.payloadRetentionDays", 1),
        ("delete_batch_size", "scheduledTasks.config.requestRecordCleanup.deleteBatchSize", 1),
        ("compress_batch_size", "scheduledTasks.config.requestRecordCleanup.compressBatchSize", 1),
        ("max_runtime_seconds", "scheduledTasks.config.requestRecordCleanup.maxRuntimeSeconds", 1),
        ("batch_sleep_ms", "scheduledTasks.config.requestRecordCleanup.batchSleepMs", 0),
        (
            "statement_timeout_seconds",
            "scheduledTasks.config.requestRecordCleanup.statementTimeoutSeconds",
            1,
        ),
        ("lock_timeout_seconds", "scheduledTasks.config.requestRecordCleanup.lockTimeoutSeconds", 1),
    ])
}

fn validate_cleanup_config(config: &TaskConfigValue) -> SchedulerResult<()> {
    for (key, min) in cleanup_integer_specs() {
        validate_positive_integer(config, key, min)?;
    }
    Ok(())
}

fn cleanup_options(now: time::OffsetDateTime, config: &TaskConfigValue) -> SchedulerResult<RequestRecordCleanupOptions> {
    Ok(RequestRecordCleanupOptions {
        record_cutoff: now - time::Duration::days(integer_config(config, "record_retention_days")?),
        payload_cutoff: now - time::Duration::days(integer_config(config, "payload_retention_days")?),
        delete_batch_size: unsigned_config(config, "delete_batch_size")?,
        compress_batch_size: unsigned_config(config, "compress_batch_size")?,
        max_runtime: Duration::from_secs(unsigned_config(config, "max_runtime_seconds")?),
        batch_sleep: Duration::from_millis(unsigned_config(config, "batch_sleep_ms")?),
        statement_timeout_seconds: integer_config(config, "statement_timeout_seconds")?,
        lock_timeout_seconds: integer_config(config, "lock_timeout_seconds")?,
    })
}

fn cleanup_message(result: RequestRecordCleanupResult) -> String {
    format!(
        "deleted_records={}, deleted_candidates={}, compressed_records={}, compressed_candidates={}, time_budget_exhausted={}",
        result.deleted_records, result.deleted_candidates, result.compressed_records, result.compressed_candidates, result.time_budget_exhausted
    )
}

fn partition_options(now: time::OffsetDateTime, config: &TaskConfigValue) -> SchedulerResult<RequestPartitionMaintenanceOptions> {
    Ok(RequestPartitionMaintenanceOptions {
        now,
        record_retention_days: integer_config(config, "record_retention_days")?,
        payload_retention_days: integer_config(config, "payload_retention_days")?,
        future_days: integer_config(config, "future_days")?,
    })
}

fn partition_message(result: RequestPartitionMaintenanceResult) -> String {
    format!(
        "created_partitions={}, dropped_partitions={}",
        result.created_partitions, result.dropped_partitions
    )
}

fn cleanup_integer_specs() -> [(&'static str, i64); 8] {
    [
        ("record_retention_days", 1),
        ("payload_retention_days", 1),
        ("delete_batch_size", 1),
        ("compress_batch_size", 1),
        ("max_runtime_seconds", 1),
        ("batch_sleep_ms", 0),
        ("statement_timeout_seconds", 1),
        ("lock_timeout_seconds", 1),
    ]
}
