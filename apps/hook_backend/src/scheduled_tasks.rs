use std::sync::Arc;

mod model_status;
mod performance_monitoring;
mod provider_quick_import;
mod recharge;
mod request_payload;
mod request_record;
#[cfg(test)]
mod tests;

use api_token::application::ApiTokenRepository;
use api_token::infra::StorageApiTokenRepository;
use scheduler::runtime::{
    DurationExt, ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerRegistry, SchedulerResult, TaskConfigValue, TaskResult,
};
use storage::scheduler::task_definition;
use types::scheduler::ScheduledTaskConfigValueType;

use self::{
    model_status::{ModelStatusCheckDispatchTask, ModelStatusRunsCleanupTask},
    performance_monitoring::{PerformanceMonitoringCleanupTask, PerformanceMonitoringSnapshotTask},
    provider_quick_import::ProviderQuickImportSyncTask,
    recharge::{RechargeOrderExpireTask, RechargePaymentPollTask},
    request_payload::{RequestPayloadBackfillTask, RequestPayloadStaleSweepTask},
    request_record::{RequestRecordCleanupTask, RequestRecordPartitionMaintenanceTask, RequestRecordStaleSweepTask},
};
use crate::{llm_proxy::LlmProxyCache, performance_monitoring_os::PerformanceOsCollector, proxy_cache_hooks::CachedApiTokenRepository};

pub fn scheduler_registry(
    cache: LlmProxyCache,
    performance_os_collector: Arc<PerformanceOsCollector>,
    recharge_service: Arc<dyn ::recharge::application::RechargeUseCase>,
    model_status_service: Arc<dyn ::model_status::application::ModelStatusUseCase>,
    provider_service: Arc<dyn ::provider::application::ProviderUseCase>,
) -> SchedulerResult<SchedulerRegistry> {
    let mut registry = SchedulerRegistry::new();
    registry.register(ApiTokenCleanupTask { cache })?;
    registry.register(RequestRecordCleanupTask)?;
    registry.register(RequestRecordStaleSweepTask)?;
    registry.register(RequestRecordPartitionMaintenanceTask)?;
    registry.register(RequestPayloadBackfillTask)?;
    registry.register(RequestPayloadStaleSweepTask)?;
    registry.register(RechargeOrderExpireTask {
        recharge_service: recharge_service.clone(),
    })?;
    registry.register(RechargePaymentPollTask { recharge_service })?;
    registry.register(PerformanceMonitoringSnapshotTask {
        os_collector: performance_os_collector,
    })?;
    registry.register(PerformanceMonitoringCleanupTask)?;
    registry.register(ModelStatusCheckDispatchTask { model_status_service })?;
    registry.register(ModelStatusRunsCleanupTask)?;
    registry.register(ProviderQuickImportSyncTask { provider_service })?;
    Ok(registry)
}

#[derive(Clone)]
struct ApiTokenCleanupTask {
    cache: LlmProxyCache,
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for ApiTokenCleanupTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "api_token_cleanup",
            "scheduledTasks.definitions.apiTokenCleanup.name",
            "scheduledTasks.definitions.apiTokenCleanup.description",
            5_i64.minutes(),
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

pub(super) fn unsigned_config(config: &TaskConfigValue, key: &str) -> SchedulerResult<u64> {
    let value = integer_config(config, key)?;
    u64::try_from(value).map_err(|_| SchedulerError::InvalidInput(format!("{key} must be greater than or equal to 0")))
}

fn api_token_error(error: api_token::application::ApiTokenError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

pub(super) fn storage_error(error: storage::StorageError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

pub(super) fn performance_collector_error(error: crate::performance_monitoring_os::PerformanceOsCollectorError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

pub(super) fn model_status_error(error: ::model_status::application::ModelStatusError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}
