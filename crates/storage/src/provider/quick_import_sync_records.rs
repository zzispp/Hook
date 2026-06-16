use sea_orm::ActiveValue;
use time::format_description::well_known::Rfc3339;
use types::provider::{
    ProviderQuickImportAnomalyActions, ProviderQuickImportCostSyncMode, ProviderQuickImportFetchFailureAction, ProviderQuickImportGroupChangedAction,
    ProviderQuickImportKeySyncInfo, ProviderQuickImportSourceKind, ProviderQuickImportSyncConfig, ProviderQuickImportSyncStatus,
    ProviderQuickImportUpstreamAnomalyAction,
};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderQuickImportSourceRecord, ProviderQuickImportSyncKeyModelRecord, ProviderQuickImportSyncKeyRecord,
    record::{provider_quick_import_keys, provider_quick_import_sources},
};

pub(super) fn sync_key_record(
    record: provider_quick_import_keys::Model,
    local_key_name: String,
    model_mappings: Vec<ProviderQuickImportSyncKeyModelRecord>,
) -> StorageResult<ProviderQuickImportSyncKeyRecord> {
    let statuses = json::decode_required(record.sync_statuses)?;
    Ok(ProviderQuickImportSyncKeyRecord {
        provider_id: record.provider_id,
        source_id: record.source_id,
        key_id: record.key_id,
        local_key_name,
        upstream_token_id: record.upstream_token_id,
        upstream_token_name: record.upstream_token_name,
        upstream_group: record.upstream_group,
        upstream_group_ratio: record.upstream_group_ratio,
        effective_cost_multiplier: record.effective_cost_multiplier,
        statuses,
        model_mappings,
    })
}

pub(super) fn key_sync_info(
    source: &ProviderQuickImportSourceRecord,
    record: provider_quick_import_keys::Model,
) -> StorageResult<ProviderQuickImportKeySyncInfo> {
    let statuses = visible_statuses(source, &record)?;
    Ok(ProviderQuickImportKeySyncInfo {
        source_kind: ProviderQuickImportSourceKind::try_from(source.source_kind.as_str()).map_err(StorageError::Database)?,
        upstream_token_id: record.upstream_token_id,
        upstream_group: record.upstream_group,
        upstream_group_ratio: record.upstream_group_ratio,
        effective_cost_multiplier: record.effective_cost_multiplier,
        statuses,
        last_synced_at: record.last_synced_at.map(format_timestamp).transpose()?,
        last_error: record.last_sync_error.or_else(|| source.last_error.clone()),
    })
}

pub(super) fn source_record(record: provider_quick_import_sources::Model, provider_name: String) -> StorageResult<ProviderQuickImportSourceRecord> {
    let sync_config = sync_config(&record)?;
    let last_status = record
        .last_status
        .as_deref()
        .map(ProviderQuickImportSyncStatus::try_from)
        .transpose()
        .map_err(StorageError::Database)?;
    Ok(ProviderQuickImportSourceRecord {
        id: record.id,
        provider_id: record.provider_id,
        provider_name,
        source_kind: record.source_kind,
        base_url: record.base_url,
        encrypted_system_access_token: record.encrypted_system_access_token,
        user_id: record.user_id,
        recharge_multiplier: record.recharge_multiplier,
        sync_config,
        last_status,
        last_error: record.last_error,
        last_synced_at: record.last_synced_at,
        consecutive_failures: record.consecutive_failures.max(0) as u32,
    })
}

pub(super) fn active_consecutive_failures(active: &provider_quick_import_sources::ActiveModel) -> i32 {
    match &active.consecutive_failures {
        ActiveValue::Set(value) | ActiveValue::Unchanged(value) => *value,
        ActiveValue::NotSet => 0,
    }
}

fn visible_statuses(source: &ProviderQuickImportSourceRecord, record: &provider_quick_import_keys::Model) -> StorageResult<Vec<ProviderQuickImportSyncStatus>> {
    if !source.sync_config.auto_sync_enabled {
        return Ok(vec![ProviderQuickImportSyncStatus::SyncDisabled]);
    }
    if let Some(status) = source.last_status
        && status == ProviderQuickImportSyncStatus::SourceFetchFailed
    {
        return Ok(vec![status]);
    }
    json::decode_required(record.sync_statuses.clone())
}

fn sync_config(record: &provider_quick_import_sources::Model) -> StorageResult<ProviderQuickImportSyncConfig> {
    Ok(ProviderQuickImportSyncConfig {
        auto_sync_enabled: record.auto_sync_enabled,
        cost_sync_mode: ProviderQuickImportCostSyncMode::try_from(record.cost_sync_mode.as_str()).map_err(StorageError::Database)?,
        anomaly_actions: ProviderQuickImportAnomalyActions {
            token_deleted: ProviderQuickImportUpstreamAnomalyAction::try_from(record.token_deleted_action.as_str()).map_err(StorageError::Database)?,
            token_disabled: ProviderQuickImportUpstreamAnomalyAction::try_from(record.token_disabled_action.as_str()).map_err(StorageError::Database)?,
            group_removed: ProviderQuickImportUpstreamAnomalyAction::try_from(record.group_removed_action.as_str()).map_err(StorageError::Database)?,
            group_changed: ProviderQuickImportGroupChangedAction::try_from(record.group_changed_action.as_str()).map_err(StorageError::Database)?,
            key_unavailable: ProviderQuickImportUpstreamAnomalyAction::try_from(record.key_unavailable_action.as_str()).map_err(StorageError::Database)?,
            model_removed: ProviderQuickImportUpstreamAnomalyAction::try_from(record.model_removed_action.as_str()).map_err(StorageError::Database)?,
        },
        fetch_failure_action: ProviderQuickImportFetchFailureAction::try_from(record.fetch_failure_action.as_str()).map_err(StorageError::Database)?,
        fetch_failure_disable_threshold: record.fetch_failure_disable_threshold.max(0) as u32,
    })
}

fn format_timestamp(value: time::OffsetDateTime) -> StorageResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| StorageError::Database(format!("quick import sync timestamp format failed: {error}")))
}
