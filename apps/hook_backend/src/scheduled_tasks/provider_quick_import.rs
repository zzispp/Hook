use std::{sync::Arc, time::Duration};

use ::provider::application::{ProviderQuickImportSyncRunOptions, ProviderQuickImportTokenRefreshRunOptions, ProviderUseCase};
use scheduler::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerResult, TaskConfigValue, TaskResult};
use storage::scheduler::task_definition;
use types::scheduler::ScheduledTaskDefinition;

use super::{integer_config, integer_fields, validate_positive_integer};

const SYNC_INTERVAL_SECONDS: i64 = 600;
const SYNC_LEASE_SECONDS: i64 = 1800;
const SYNC_BATCH_SIZE: i64 = 20;
const SYNC_MAX_RUNTIME_SECONDS: i64 = 300;
const TOKEN_REFRESH_INTERVAL_SECONDS: i64 = 300;
const TOKEN_REFRESH_LEASE_SECONDS: i64 = 900;
const TOKEN_REFRESH_BATCH_SIZE: i64 = 20;
const TOKEN_REFRESH_THRESHOLD_MINUTES: i64 = 60;
const TOKEN_REFRESH_MAX_RUNTIME_SECONDS: i64 = 300;

#[derive(Clone)]
pub(super) struct ProviderQuickImportSyncTask {
    pub(super) provider_service: Arc<dyn ProviderUseCase>,
}

#[derive(Clone)]
pub(super) struct ProviderQuickImportSub2apiTokenRefreshTask {
    pub(super) provider_service: Arc<dyn ProviderUseCase>,
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for ProviderQuickImportSyncTask {
    fn definition(&self) -> ScheduledTaskDefinition {
        provider_quick_import_sync_definition()
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_provider_quick_import_sync_config(config)
    }

    async fn run(&self, _ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let batch_size = u64::try_from(integer_config(&config, "batch_size")?).map_err(integer_error)?;
        let timeout_seconds = u64::try_from(integer_config(&config, "max_runtime_seconds")?).map_err(integer_error)?;
        let report = tokio::time::timeout(
            Duration::from_secs(timeout_seconds),
            self.provider_service
                .run_quick_import_sync(ProviderQuickImportSyncRunOptions { limit: batch_size }),
        )
        .await
        .map_err(|_| SchedulerError::Infrastructure("provider quick import sync timed out".into()))?
        .map_err(provider_error)?;
        Ok(Some(format!(
            "scanned_count={}, synced_count={}, failed_count={}, disabled_key_count={}, updated_cost_count={}",
            report.scanned_count, report.synced_count, report.failed_count, report.disabled_key_count, report.updated_cost_count
        )))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for ProviderQuickImportSub2apiTokenRefreshTask {
    fn definition(&self) -> ScheduledTaskDefinition {
        provider_quick_import_sub2api_token_refresh_definition()
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_provider_quick_import_sub2api_token_refresh_config(config)
    }

    async fn run(&self, _ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let batch_size = u64::try_from(integer_config(&config, "batch_size")?).map_err(integer_error)?;
        let refresh_threshold_minutes = integer_config(&config, "refresh_threshold_minutes")?;
        let timeout_seconds = u64::try_from(integer_config(&config, "max_runtime_seconds")?).map_err(integer_error)?;
        let report = tokio::time::timeout(
            Duration::from_secs(timeout_seconds),
            self.provider_service.run_quick_import_token_refresh(ProviderQuickImportTokenRefreshRunOptions {
                limit: batch_size,
                refresh_threshold_minutes,
            }),
        )
        .await
        .map_err(|_| SchedulerError::Infrastructure("provider quick import sub2api token refresh timed out".into()))?
        .map_err(provider_error)?;
        Ok(Some(format!(
            "scanned_count={}, refreshed_count={}, skipped_count={}, failed_count={}",
            report.scanned_count, report.refreshed_count, report.skipped_count, report.failed_count
        )))
    }
}

pub(super) fn provider_quick_import_sync_definition() -> ScheduledTaskDefinition {
    task_definition(
        "provider_quick_import_sync",
        "scheduledTasks.definitions.providerQuickImportSync.name",
        "scheduledTasks.definitions.providerQuickImportSync.description",
        SYNC_INTERVAL_SECONDS,
        SYNC_LEASE_SECONDS,
        serde_json::json!({
            "batch_size": SYNC_BATCH_SIZE,
            "max_runtime_seconds": SYNC_MAX_RUNTIME_SECONDS
        }),
        integer_fields(&[
            ("batch_size", "scheduledTasks.config.providerQuickImportSync.batchSize", 1),
            ("max_runtime_seconds", "scheduledTasks.config.providerQuickImportSync.maxRuntimeSeconds", 1),
        ]),
    )
}

pub(super) fn provider_quick_import_sub2api_token_refresh_definition() -> ScheduledTaskDefinition {
    task_definition(
        "provider_quick_import_sub2api_token_refresh",
        "scheduledTasks.definitions.providerQuickImportSub2apiTokenRefresh.name",
        "scheduledTasks.definitions.providerQuickImportSub2apiTokenRefresh.description",
        TOKEN_REFRESH_INTERVAL_SECONDS,
        TOKEN_REFRESH_LEASE_SECONDS,
        serde_json::json!({
            "batch_size": TOKEN_REFRESH_BATCH_SIZE,
            "refresh_threshold_minutes": TOKEN_REFRESH_THRESHOLD_MINUTES,
            "max_runtime_seconds": TOKEN_REFRESH_MAX_RUNTIME_SECONDS
        }),
        integer_fields(&[
            ("batch_size", "scheduledTasks.config.providerQuickImportSub2apiTokenRefresh.batchSize", 1),
            (
                "refresh_threshold_minutes",
                "scheduledTasks.config.providerQuickImportSub2apiTokenRefresh.refreshThresholdMinutes",
                1,
            ),
            (
                "max_runtime_seconds",
                "scheduledTasks.config.providerQuickImportSub2apiTokenRefresh.maxRuntimeSeconds",
                1,
            ),
        ]),
    )
}

pub(super) fn validate_provider_quick_import_sync_config(config: &TaskConfigValue) -> SchedulerResult<()> {
    validate_positive_integer(config, "batch_size", 1)?;
    validate_positive_integer(config, "max_runtime_seconds", 1)
}

pub(super) fn validate_provider_quick_import_sub2api_token_refresh_config(config: &TaskConfigValue) -> SchedulerResult<()> {
    validate_positive_integer(config, "batch_size", 1)?;
    validate_positive_integer(config, "refresh_threshold_minutes", 1)?;
    validate_positive_integer(config, "max_runtime_seconds", 1)
}

fn integer_error(_error: std::num::TryFromIntError) -> SchedulerError {
    SchedulerError::InvalidInput("integer config must be greater than or equal to 0".into())
}

fn provider_error(error: ::provider::application::ProviderError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}
