use std::sync::Arc;

use api_token::application::ApiTokenRepository;
use api_token::infra::StorageApiTokenRepository;
use scheduler::runtime::{
    DurationExt, ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerRegistry, SchedulerResult, TaskConfigValue, TaskResult,
};
use storage::{
    performance_monitoring::{PerformanceMonitoringStore, SnapshotAggregationWindow},
    provider::ProviderStore,
    scheduler::task_definition,
};
use types::performance_monitoring::SnapshotGranularity;
use types::scheduler::ScheduledTaskConfigValueType;

use crate::{llm_proxy::LlmProxyCache, performance_monitoring_os::PerformanceOsCollector, proxy_cache_hooks::CachedApiTokenRepository};

const REQUEST_RECORD_STALE_SWEEP_INTERVAL_SECONDS: i64 = 300;
const REQUEST_RECORD_STALE_PENDING_TIMEOUT_MINUTES: i64 = 15;
const REQUEST_RECORD_STALE_STREAMING_TIMEOUT_MINUTES: i64 = 120;

pub fn scheduler_registry(cache: LlmProxyCache, performance_os_collector: Arc<PerformanceOsCollector>) -> SchedulerResult<SchedulerRegistry> {
    let mut registry = SchedulerRegistry::new();
    registry.register(ApiTokenCleanupTask { cache })?;
    registry.register(RequestRecordCleanupTask)?;
    registry.register(RequestRecordStaleSweepTask)?;
    registry.register(PerformanceMonitoringSnapshotTask {
        os_collector: performance_os_collector,
    })?;
    registry.register(PerformanceMonitoringCleanupTask)?;
    Ok(registry)
}

#[derive(Clone)]
struct ApiTokenCleanupTask {
    cache: LlmProxyCache,
}

#[derive(Clone, Copy)]
struct RequestRecordCleanupTask;

#[derive(Clone, Copy)]
struct RequestRecordStaleSweepTask;

#[derive(Clone)]
struct PerformanceMonitoringSnapshotTask {
    os_collector: Arc<PerformanceOsCollector>,
}

#[derive(Clone, Copy)]
struct PerformanceMonitoringCleanupTask;

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for ApiTokenCleanupTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "api_token_cleanup",
            "scheduledTasks.definitions.apiTokenCleanup.name",
            "scheduledTasks.definitions.apiTokenCleanup.description",
            5_i64.minutes(),
            serde_json::json!({}),
            Vec::new(),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_empty_config(config)
    }

    async fn run(&self, ctx: ScheduleTaskContext, _config: TaskConfigValue) -> TaskResult {
        let deleted = CachedApiTokenRepository::new(StorageApiTokenRepository::new(ctx.database), self.cache.clone())
            .delete_expired_tokens()
            .await
            .map_err(api_token_error)?;
        Ok(Some(format!("deleted_tokens={deleted}")))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RequestRecordCleanupTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "request_record_cleanup",
            "scheduledTasks.definitions.requestRecordCleanup.name",
            "scheduledTasks.definitions.requestRecordCleanup.description",
            24_i64.hours(),
            serde_json::json!({
                "record_retention_days": 365,
                "payload_retention_days": 30
            }),
            integer_fields(&[
                ("record_retention_days", "scheduledTasks.config.requestRecordCleanup.recordRetentionDays", 1),
                ("payload_retention_days", "scheduledTasks.config.requestRecordCleanup.payloadRetentionDays", 1),
            ]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "record_retention_days", 1)?;
        validate_positive_integer(config, "payload_retention_days", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let now = time::OffsetDateTime::now_utc();
        let record_retention_days = integer_config(&config, "record_retention_days")?;
        let payload_retention_days = integer_config(&config, "payload_retention_days")?;
        let store = ProviderStore::new(ctx.database);
        let deleted_records = store
            .delete_request_records_before(now - time::Duration::days(record_retention_days))
            .await
            .map_err(storage_error)?;
        let compressed_payloads = store
            .compress_request_record_payloads_before(now - time::Duration::days(payload_retention_days))
            .await
            .map_err(storage_error)?;
        Ok(Some(format!("deleted_records={deleted_records}, compressed_payloads={compressed_payloads}")))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RequestRecordStaleSweepTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "request_record_stale_sweep",
            "scheduledTasks.definitions.requestRecordStaleSweep.name",
            "scheduledTasks.definitions.requestRecordStaleSweep.description",
            REQUEST_RECORD_STALE_SWEEP_INTERVAL_SECONDS,
            serde_json::json!({
                "stale_pending_timeout_minutes": REQUEST_RECORD_STALE_PENDING_TIMEOUT_MINUTES,
                "stale_streaming_timeout_minutes": REQUEST_RECORD_STALE_STREAMING_TIMEOUT_MINUTES
            }),
            integer_fields(&[
                (
                    "stale_pending_timeout_minutes",
                    "scheduledTasks.config.requestRecordStaleSweep.pendingTimeoutMinutes",
                    1,
                ),
                (
                    "stale_streaming_timeout_minutes",
                    "scheduledTasks.config.requestRecordStaleSweep.streamingTimeoutMinutes",
                    1,
                ),
            ]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "stale_pending_timeout_minutes", 1)?;
        validate_positive_integer(config, "stale_streaming_timeout_minutes", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let now = time::OffsetDateTime::now_utc();
        let pending_timeout = integer_config(&config, "stale_pending_timeout_minutes")?;
        let streaming_timeout = integer_config(&config, "stale_streaming_timeout_minutes")?;
        let report = ProviderStore::new(ctx.database)
            .sweep_stale_request_records(now - time::Duration::minutes(pending_timeout), now - time::Duration::minutes(streaming_timeout))
            .await
            .map_err(storage_error)?;
        Ok(Some(format!(
            "pending_records={}, streaming_records={}, failed_candidates={}, skipped_candidates={}",
            report.pending_records, report.streaming_records, report.failed_candidates, report.skipped_candidates
        )))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for PerformanceMonitoringSnapshotTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "performance_monitoring_snapshot",
            "scheduledTasks.definitions.performanceMonitoringSnapshot.name",
            "scheduledTasks.definitions.performanceMonitoringSnapshot.description",
            60,
            serde_json::json!({}),
            Vec::new(),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_empty_config(config)
    }

    async fn run(&self, ctx: ScheduleTaskContext, _config: TaskConfigValue) -> TaskResult {
        let now = time::OffsetDateTime::now_utc();
        let windows = aggregation_windows(now);
        let count = windows.len();
        let system = self.os_collector.clone().snapshot().await.map_err(performance_collector_error)?;
        let store = PerformanceMonitoringStore::new(ctx.database);
        for window in windows {
            store.aggregate_window_with_system(window, system.clone()).await.map_err(storage_error)?;
        }
        Ok(Some(format!("buckets={count}")))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for PerformanceMonitoringCleanupTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "performance_monitoring_cleanup",
            "scheduledTasks.definitions.performanceMonitoringCleanup.name",
            "scheduledTasks.definitions.performanceMonitoringCleanup.description",
            24_i64.hours(),
            serde_json::json!({
                "retention_days": 30
            }),
            integer_fields(&[("retention_days", "scheduledTasks.config.performanceMonitoringCleanup.retentionDays", 1)]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "retention_days", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let retention_days = integer_config(&config, "retention_days")?;
        let deleted = PerformanceMonitoringStore::new(ctx.database)
            .delete_snapshots_before(time::OffsetDateTime::now_utc() - time::Duration::days(retention_days))
            .await
            .map_err(storage_error)?;
        Ok(Some(format!("deleted_snapshots={deleted}")))
    }
}

fn integer_fields(items: &[(&str, &str, i64)]) -> Vec<types::scheduler::ScheduledTaskConfigField> {
    items
        .iter()
        .map(|(key, label_key, min)| types::scheduler::ScheduledTaskConfigField {
            key: (*key).to_owned(),
            label_key: (*label_key).to_owned(),
            value_type: ScheduledTaskConfigValueType::Integer,
            required: true,
            min: Some(*min),
            max: None,
            unit_key: None,
        })
        .collect()
}

fn validate_empty_config(config: &TaskConfigValue) -> SchedulerResult<()> {
    if config.is_null() || config == &serde_json::json!({}) {
        return Ok(());
    }
    Err(SchedulerError::InvalidInput("task config must be an empty object".into()))
}

fn validate_positive_integer(config: &TaskConfigValue, key: &str, min: i64) -> SchedulerResult<()> {
    let value = integer_config(config, key)?;
    if value < min {
        return Err(SchedulerError::InvalidInput(format!("{key} must be greater than or equal to {min}")));
    }
    Ok(())
}

fn integer_config(config: &TaskConfigValue, key: &str) -> SchedulerResult<i64> {
    config
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| SchedulerError::InvalidInput(format!("missing integer config field: {key}")))
}

fn aggregation_windows(now: time::OffsetDateTime) -> Vec<SnapshotAggregationWindow> {
    let minute_start = floor_minute(now) - time::Duration::minutes(1);
    let mut windows = vec![window(SnapshotGranularity::Minute, minute_start)];
    if minute_start.minute() == 59 {
        windows.push(window(SnapshotGranularity::Hour, floor_hour(now) - time::Duration::hours(1)));
    }
    if minute_start.hour() == 23 && minute_start.minute() == 59 {
        windows.push(window(SnapshotGranularity::Day, floor_day(now) - time::Duration::days(1)));
    }
    windows
}

fn window(granularity: SnapshotGranularity, started_at: time::OffsetDateTime) -> SnapshotAggregationWindow {
    SnapshotAggregationWindow {
        granularity,
        started_at,
        ended_at: started_at + time::Duration::seconds(granularity.bucket_seconds()),
    }
}

fn floor_minute(value: time::OffsetDateTime) -> time::OffsetDateTime {
    value.replace_second(0).unwrap().replace_nanosecond(0).unwrap()
}

fn floor_hour(value: time::OffsetDateTime) -> time::OffsetDateTime {
    floor_minute(value).replace_minute(0).unwrap()
}

fn floor_day(value: time::OffsetDateTime) -> time::OffsetDateTime {
    floor_hour(value).replace_hour(0).unwrap()
}

fn api_token_error(error: api_token::application::ApiTokenError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

fn storage_error(error: storage::StorageError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

fn performance_collector_error(error: crate::performance_monitoring_os::PerformanceOsCollectorError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}
