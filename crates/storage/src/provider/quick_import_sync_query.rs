use std::collections::BTreeMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use time::format_description::well_known::Rfc3339;
use types::provider::{
    ProviderQuickImportAnomalyActions, ProviderQuickImportCostSyncMode, ProviderQuickImportFetchFailureAction, ProviderQuickImportGroupChangedAction,
    ProviderQuickImportKeySyncInfo, ProviderQuickImportSourceKind, ProviderQuickImportSyncConfig, ProviderQuickImportSyncStatus,
    ProviderQuickImportUpstreamAnomalyAction,
};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderQuickImportSourceRecord, ProviderQuickImportSourceRecordPatch, ProviderQuickImportSyncKeyModelRecord, ProviderQuickImportSyncKeyRecord,
    ProviderQuickImportSyncKeyRecordPatch, ProviderStore,
    record::{provider_quick_import_key_models, provider_quick_import_keys, provider_quick_import_sources},
};

pub async fn key_sync_info_by_provider(store: &ProviderStore, provider_id: &str) -> StorageResult<BTreeMap<String, ProviderQuickImportKeySyncInfo>> {
    let Some(source) = source_for_provider(store, provider_id).await? else {
        return Ok(BTreeMap::new());
    };
    let records = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(provider_id))
        .all(store.connection())
        .await?;
    records
        .into_iter()
        .map(|record| Ok((record.key_id.clone(), key_sync_info(&source, record)?)))
        .collect()
}

pub async fn source_for_provider(store: &ProviderStore, provider_id: &str) -> StorageResult<Option<ProviderQuickImportSourceRecord>> {
    provider_quick_import_sources::Entity::find()
        .filter(provider_quick_import_sources::Column::ProviderId.eq(provider_id))
        .one(store.connection())
        .await?
        .map(source_record)
        .transpose()
}

pub async fn list_sources(store: &ProviderStore, limit: u64) -> StorageResult<Vec<ProviderQuickImportSourceRecord>> {
    let records = provider_quick_import_sources::Entity::find()
        .filter(provider_quick_import_sources::Column::AutoSyncEnabled.eq(true))
        .order_by_asc(provider_quick_import_sources::Column::LastSyncedAt)
        .limit(limit)
        .all(store.connection())
        .await?;
    records.into_iter().map(source_record).collect()
}

pub async fn keys_for_source(store: &ProviderStore, source_id: &str) -> StorageResult<Vec<ProviderQuickImportSyncKeyRecord>> {
    let keys = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::SourceId.eq(source_id))
        .all(store.connection())
        .await?;
    let models = key_models_by_key(store, source_id).await?;
    keys.into_iter()
        .map(|key| {
            let model_mappings = models.get(key.key_id.as_str()).cloned().unwrap_or_default();
            sync_key_record(key, model_mappings)
        })
        .collect()
}

pub async fn key_for_provider_key(store: &ProviderStore, provider_id: &str, key_id: &str) -> StorageResult<Option<ProviderQuickImportSyncKeyRecord>> {
    let Some(key) = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_quick_import_keys::Column::KeyId.eq(key_id))
        .one(store.connection())
        .await?
    else {
        return Ok(None);
    };
    let models = provider_quick_import_key_models::Entity::find()
        .filter(provider_quick_import_key_models::Column::ProviderId.eq(provider_id))
        .filter(provider_quick_import_key_models::Column::KeyId.eq(key_id))
        .all(store.connection())
        .await?
        .into_iter()
        .map(|record| ProviderQuickImportSyncKeyModelRecord {
            upstream_model_id: record.upstream_model_id,
            global_model_id: record.global_model_id,
        })
        .collect();
    sync_key_record(key, models).map(Some)
}

pub async fn update_source(
    store: &ProviderStore,
    provider_id: &str,
    patch: ProviderQuickImportSourceRecordPatch,
) -> StorageResult<ProviderQuickImportSourceRecord> {
    let Some(record) = provider_quick_import_sources::Entity::find()
        .filter(provider_quick_import_sources::Column::ProviderId.eq(provider_id))
        .one(store.connection())
        .await?
    else {
        return Err(StorageError::NotFound);
    };
    let mut active: provider_quick_import_sources::ActiveModel = record.into();
    apply_source_patch(&mut active, patch);
    source_record(active.update(store.connection()).await?)
}

pub async fn update_source_run(
    store: &ProviderStore,
    source_id: &str,
    status: Option<ProviderQuickImportSyncStatus>,
    error: Option<String>,
    failed: bool,
) -> StorageResult<()> {
    let record = provider_quick_import_sources::Entity::find_by_id(source_id.to_owned())
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    let now = time::OffsetDateTime::now_utc();
    let mut active: provider_quick_import_sources::ActiveModel = record.into();
    active.last_status = Set(status.map(|value| value.as_str().to_owned()));
    active.last_error = Set(error);
    active.last_synced_at = Set(Some(now));
    active.consecutive_failures = Set(if failed { active_consecutive_failures(&active) + 1 } else { 0 });
    active.updated_at = Set(now);
    active.update(store.connection()).await?;
    Ok(())
}

pub async fn update_keys(store: &ProviderStore, provider_id: &str, patches: Vec<ProviderQuickImportSyncKeyRecordPatch>) -> StorageResult<()> {
    for patch in patches {
        update_key(store, provider_id, patch).await?;
    }
    Ok(())
}

fn apply_source_patch(active: &mut provider_quick_import_sources::ActiveModel, patch: ProviderQuickImportSourceRecordPatch) {
    if let Some(base_url) = patch.base_url {
        active.base_url = Set(base_url);
    }
    if let Some(token) = patch.encrypted_system_access_token {
        active.encrypted_system_access_token = Set(token);
    }
    if let Some(user_id) = patch.user_id {
        active.user_id = Set(user_id);
    }
    if let Some(multiplier) = patch.recharge_multiplier {
        active.recharge_multiplier = Set(multiplier);
    }
    if let Some(config) = patch.sync_config {
        active.auto_sync_enabled = Set(config.auto_sync_enabled);
        active.cost_sync_mode = Set(config.cost_sync_mode.as_str().to_owned());
        active.token_deleted_action = Set(config.anomaly_actions.token_deleted.as_str().to_owned());
        active.token_disabled_action = Set(config.anomaly_actions.token_disabled.as_str().to_owned());
        active.group_removed_action = Set(config.anomaly_actions.group_removed.as_str().to_owned());
        active.group_changed_action = Set(config.anomaly_actions.group_changed.as_str().to_owned());
        active.key_unavailable_action = Set(config.anomaly_actions.key_unavailable.as_str().to_owned());
        active.model_removed_action = Set(config.anomaly_actions.model_removed.as_str().to_owned());
        active.fetch_failure_action = Set(config.fetch_failure_action.as_str().to_owned());
        active.fetch_failure_disable_threshold = Set(config.fetch_failure_disable_threshold as i32);
    }
    active.updated_at = Set(time::OffsetDateTime::now_utc());
}

async fn update_key(store: &ProviderStore, provider_id: &str, patch: ProviderQuickImportSyncKeyRecordPatch) -> StorageResult<()> {
    let record = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_quick_import_keys::Column::KeyId.eq(&patch.key_id))
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: provider_quick_import_keys::ActiveModel = record.into();
    if let Some(value) = patch.upstream_group {
        active.upstream_group = Set(value);
    }
    if let Some(value) = patch.upstream_group_ratio {
        active.upstream_group_ratio = Set(value);
    }
    if let Some(value) = patch.effective_cost_multiplier {
        active.effective_cost_multiplier = Set(value);
    }
    active.sync_statuses = Set(json::encode_required(&patch.statuses)?);
    active.last_sync_error = Set(patch.last_error);
    active.last_synced_at = Set(Some(time::OffsetDateTime::now_utc()));
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(store.connection()).await?;
    Ok(())
}

async fn key_models_by_key(store: &ProviderStore, source_id: &str) -> StorageResult<BTreeMap<String, Vec<ProviderQuickImportSyncKeyModelRecord>>> {
    let records = provider_quick_import_key_models::Entity::find()
        .filter(provider_quick_import_key_models::Column::SourceId.eq(source_id))
        .all(store.connection())
        .await?;
    let mut output: BTreeMap<String, Vec<ProviderQuickImportSyncKeyModelRecord>> = BTreeMap::new();
    for record in records {
        output.entry(record.key_id).or_default().push(ProviderQuickImportSyncKeyModelRecord {
            upstream_model_id: record.upstream_model_id,
            global_model_id: record.global_model_id,
        });
    }
    Ok(output)
}

fn sync_key_record(
    record: provider_quick_import_keys::Model,
    model_mappings: Vec<ProviderQuickImportSyncKeyModelRecord>,
) -> StorageResult<ProviderQuickImportSyncKeyRecord> {
    let statuses = json::decode_required(record.sync_statuses)?;
    Ok(ProviderQuickImportSyncKeyRecord {
        provider_id: record.provider_id,
        source_id: record.source_id,
        key_id: record.key_id,
        upstream_token_id: record.upstream_token_id,
        upstream_token_name: record.upstream_token_name,
        upstream_group: record.upstream_group,
        upstream_group_ratio: record.upstream_group_ratio,
        effective_cost_multiplier: record.effective_cost_multiplier,
        statuses,
        model_mappings,
    })
}

fn active_consecutive_failures(active: &provider_quick_import_sources::ActiveModel) -> i32 {
    match &active.consecutive_failures {
        sea_orm::ActiveValue::Set(value) | sea_orm::ActiveValue::Unchanged(value) => *value,
        sea_orm::ActiveValue::NotSet => 0,
    }
}

fn key_sync_info(source: &ProviderQuickImportSourceRecord, record: provider_quick_import_keys::Model) -> StorageResult<ProviderQuickImportKeySyncInfo> {
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

fn source_record(record: provider_quick_import_sources::Model) -> StorageResult<ProviderQuickImportSourceRecord> {
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
