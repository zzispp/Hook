use std::sync::Arc;

mod model_status;
mod performance_monitoring;

use api_token::application::ApiTokenRepository;
use api_token::infra::StorageApiTokenRepository;
use scheduler::runtime::{
    DurationExt, ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerRegistry, SchedulerResult, TaskConfigValue, TaskResult,
};
use storage::{provider::ProviderStore, scheduler::task_definition};
use types::scheduler::ScheduledTaskConfigValueType;

use self::{
    model_status::{ModelStatusCheckDispatchTask, ModelStatusRunsCleanupTask},
    performance_monitoring::{PerformanceMonitoringCleanupTask, PerformanceMonitoringSnapshotTask},
};
use crate::{llm_proxy::LlmProxyCache, performance_monitoring_os::PerformanceOsCollector, proxy_cache_hooks::CachedApiTokenRepository};

const REQUEST_RECORD_STALE_SWEEP_INTERVAL_SECONDS: i64 = 300;
const REQUEST_RECORD_STALE_PENDING_TIMEOUT_MINUTES: i64 = 15;
const REQUEST_RECORD_STALE_STREAMING_TIMEOUT_MINUTES: i64 = 120;
const RECHARGE_PAYMENT_POLL_INTERVAL_SECONDS: i64 = 60;
const RECHARGE_PAYMENT_POLL_LIMIT: i64 = 50;

pub fn scheduler_registry(
    cache: LlmProxyCache,
    performance_os_collector: Arc<PerformanceOsCollector>,
    recharge_service: Arc<dyn recharge::application::RechargeUseCase>,
    model_status_service: Arc<dyn ::model_status::application::ModelStatusUseCase>,
) -> SchedulerResult<SchedulerRegistry> {
    let mut registry = SchedulerRegistry::new();
    registry.register(ApiTokenCleanupTask { cache })?;
    registry.register(RequestRecordCleanupTask)?;
    registry.register(RequestRecordStaleSweepTask)?;
    registry.register(RechargePaymentPollTask { recharge_service })?;
    registry.register(PerformanceMonitoringSnapshotTask {
        os_collector: performance_os_collector,
    })?;
    registry.register(PerformanceMonitoringCleanupTask)?;
    registry.register(ModelStatusCheckDispatchTask { model_status_service })?;
    registry.register(ModelStatusRunsCleanupTask)?;
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
struct RechargePaymentPollTask {
    recharge_service: Arc<dyn recharge::application::RechargeUseCase>,
}

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
impl ScheduledTaskLifecycle for RechargePaymentPollTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "recharge_payment_poll",
            "scheduledTasks.definitions.rechargePaymentPoll.name",
            "scheduledTasks.definitions.rechargePaymentPoll.description",
            RECHARGE_PAYMENT_POLL_INTERVAL_SECONDS,
            serde_json::json!({
                "limit": RECHARGE_PAYMENT_POLL_LIMIT
            }),
            integer_fields(&[("limit", "scheduledTasks.config.rechargePaymentPoll.limit", 1)]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "limit", 1)
    }

    async fn run(&self, _ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let limit = integer_config(&config, "limit")?;
        let limit = u64::try_from(limit).map_err(|_| SchedulerError::InvalidInput("limit must be greater than 0".into()))?;
        let result = self.recharge_service.poll_pending_payment_orders(limit).await.map_err(recharge_error)?;
        Ok(Some(format!(
            "checked={}, paid={}, unsupported={}",
            result.checked, result.paid, result.unsupported
        )))
    }
}

pub(super) fn integer_fields(items: &[(&str, &str, i64)]) -> Vec<types::scheduler::ScheduledTaskConfigField> {
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

pub(super) fn validate_empty_config(config: &TaskConfigValue) -> SchedulerResult<()> {
    if config.is_null() || config == &serde_json::json!({}) {
        return Ok(());
    }
    Err(SchedulerError::InvalidInput("task config must be an empty object".into()))
}

pub(super) fn validate_positive_integer(config: &TaskConfigValue, key: &str, min: i64) -> SchedulerResult<()> {
    let value = integer_config(config, key)?;
    if value < min {
        return Err(SchedulerError::InvalidInput(format!("{key} must be greater than or equal to {min}")));
    }
    Ok(())
}

pub(super) fn integer_config(config: &TaskConfigValue, key: &str) -> SchedulerResult<i64> {
    config
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| SchedulerError::InvalidInput(format!("missing integer config field: {key}")))
}

fn api_token_error(error: api_token::application::ApiTokenError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

pub(super) fn storage_error(error: storage::StorageError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

fn recharge_error(error: recharge::application::RechargeError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

pub(super) fn performance_collector_error(error: crate::performance_monitoring_os::PerformanceOsCollectorError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

pub(super) fn model_status_error(error: ::model_status::application::ModelStatusError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}
