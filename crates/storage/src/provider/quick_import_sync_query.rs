use std::collections::{BTreeMap, BTreeSet};

use sea_orm::{ActiveModelTrait, ActiveValue::Unchanged, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::provider::{ProviderQuickImportKeySyncInfo, ProviderQuickImportSyncStatus};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderQuickImportSourceRecord, ProviderQuickImportSourceRecordPatch, ProviderQuickImportSyncKeyModelRecord, ProviderQuickImportSyncKeyRecord,
    ProviderQuickImportSyncKeyRecordPatch, ProviderStore,
    quick_import_sync_lookup::{key_name, key_names, provider_name, provider_names, require_key_name, require_provider_name},
    quick_import_sync_records::{active_consecutive_failures, key_sync_info, source_record, sync_key_record},
    record::{provider_key_model_mappings, provider_models, provider_quick_import_keys, provider_quick_import_sources},
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
    let Some(record) = provider_quick_import_sources::Entity::find()
        .filter(provider_quick_import_sources::Column::ProviderId.eq(provider_id))
        .one(store.connection())
        .await?
    else {
        return Ok(None);
    };
    let provider_name = provider_name(store, &record.provider_id).await?;
    source_record(record, provider_name).map(Some)
}

pub async fn list_sources(store: &ProviderStore, limit: u64) -> StorageResult<Vec<ProviderQuickImportSourceRecord>> {
    let records = provider_quick_import_sources::Entity::find()
        .filter(provider_quick_import_sources::Column::AutoSyncEnabled.eq(true))
        .order_by_asc(provider_quick_import_sources::Column::LastSyncedAt)
        .limit(limit)
        .all(store.connection())
        .await?;
    let names = provider_names(store, records.iter().map(|record| record.provider_id.clone()).collect()).await?;
    records
        .into_iter()
        .map(|record| {
            let provider_name = require_provider_name(&names, &record.provider_id)?;
            source_record(record, provider_name)
        })
        .collect()
}

pub async fn list_sub2api_token_refresh_sources(store: &ProviderStore, limit: u64) -> StorageResult<Vec<ProviderQuickImportSourceRecord>> {
    let records = provider_quick_import_sources::Entity::find()
        .filter(provider_quick_import_sources::Column::SourceKind.eq("sub2api"))
        .filter(provider_quick_import_sources::Column::EncryptedAuthToken.ne(""))
        .filter(provider_quick_import_sources::Column::EncryptedRefreshToken.ne(""))
        .filter(provider_quick_import_sources::Column::TokenExpiresAt.is_not_null())
        .order_by_asc(provider_quick_import_sources::Column::TokenExpiresAt)
        .limit(limit)
        .all(store.connection())
        .await?;
    let names = provider_names(store, records.iter().map(|record| record.provider_id.clone()).collect()).await?;
    records
        .into_iter()
        .map(|record| {
            let provider_name = require_provider_name(&names, &record.provider_id)?;
            source_record(record, provider_name)
        })
        .collect()
}

pub async fn keys_for_source(store: &ProviderStore, source_id: &str) -> StorageResult<Vec<ProviderQuickImportSyncKeyRecord>> {
    let keys = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::SourceId.eq(source_id))
        .all(store.connection())
        .await?;
    let models = key_models_by_key(store, source_id).await?;
    let names = key_names(store, &keys).await?;
    keys.into_iter()
        .map(|key| {
            let local_key_name = require_key_name(&names, &key.provider_id, &key.key_id)?;
            let model_mappings = models.get(key.key_id.as_str()).cloned().unwrap_or_default();
            sync_key_record(key, local_key_name, model_mappings)
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
    let models = key_models_for_keys(store, std::slice::from_ref(&key.key_id)).await?;
    let local_key_name = key_name(store, provider_id, key_id).await?;
    sync_key_record(key.clone(), local_key_name, models.get(&key.key_id).cloned().unwrap_or_default()).map(Some)
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
    let mut active = provider_quick_import_sources::ActiveModel {
        id: Unchanged(record.id),
        ..Default::default()
    };
    apply_source_patch(&mut active, patch);
    let updated = active.update(store.connection()).await?;
    let provider_name = provider_name(store, provider_id).await?;
    source_record(updated, provider_name)
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
    if let Some(email) = patch.email {
        active.email = Set(email);
    }
    if let Some(password) = patch.encrypted_password {
        active.encrypted_password = Set(password);
    }
    if let Some(auth_token) = patch.encrypted_auth_token {
        active.encrypted_auth_token = Set(auth_token);
    }
    if let Some(refresh_token) = patch.encrypted_refresh_token {
        active.encrypted_refresh_token = Set(refresh_token);
    }
    if let Some(token_expires_at) = patch.token_expires_at {
        active.token_expires_at = Set(token_expires_at);
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
    if let Some(value) = patch.upstream_group_id {
        active.upstream_group_id = Set(value);
    }
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
    let keys = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::SourceId.eq(source_id))
        .all(store.connection())
        .await?;
    let key_ids = keys.into_iter().map(|record| record.key_id).collect::<Vec<_>>();
    key_models_for_keys(store, &key_ids).await
}

async fn key_models_for_keys(store: &ProviderStore, key_ids: &[String]) -> StorageResult<BTreeMap<String, Vec<ProviderQuickImportSyncKeyModelRecord>>> {
    if key_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_key_model_mappings::Entity::find()
        .filter(provider_key_model_mappings::Column::KeyId.is_in(key_ids.iter().cloned()))
        .all(store.connection())
        .await?;
    let provider_model_ids = records.iter().map(|record| record.provider_model_id.clone()).collect::<BTreeSet<_>>();
    let models = provider_models::Entity::find()
        .filter(provider_models::Column::Id.is_in(provider_model_ids))
        .all(store.connection())
        .await?
        .into_iter()
        .map(|record| (record.id, record.global_model_id))
        .collect::<BTreeMap<_, _>>();
    let mut output: BTreeMap<String, Vec<ProviderQuickImportSyncKeyModelRecord>> = BTreeMap::new();
    for record in records {
        let Some(global_model_id) = models.get(&record.provider_model_id).cloned() else {
            continue;
        };
        output.entry(record.key_id).or_default().push(ProviderQuickImportSyncKeyModelRecord {
            provider_model_id: record.provider_model_id,
            global_model_id,
            upstream_model_name: record.upstream_model_name,
            reasoning_effort: record.reasoning_effort,
        });
    }
    Ok(output)
}
