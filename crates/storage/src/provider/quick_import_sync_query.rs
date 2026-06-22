use std::collections::{BTreeMap, BTreeSet};

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::provider::{ProviderQuickImportKeySyncInfo, ProviderQuickImportSyncStatus};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderQuickImportSourceRecord, ProviderQuickImportSourceRecordPatch, ProviderQuickImportSyncKeyModelRecord, ProviderQuickImportSyncKeyRecord,
    ProviderQuickImportSyncKeyRecordPatch, ProviderStore,
    quick_import_sync_records::{active_consecutive_failures, key_sync_info, source_record, sync_key_record},
    record::{provider_api_keys, provider_quick_import_key_models, provider_quick_import_keys, provider_quick_import_sources, providers},
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
    let local_key_name = key_name(store, provider_id, key_id).await?;
    sync_key_record(key, local_key_name, models).map(Some)
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
    let updated = active.update(store.connection()).await?;
    let provider_name = provider_name(store, &updated.provider_id).await?;
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

async fn provider_name(store: &ProviderStore, provider_id: &str) -> StorageResult<String> {
    providers::Entity::find_by_id(provider_id.to_owned())
        .one(store.connection())
        .await?
        .map(|record| record.name)
        .ok_or_else(|| missing_provider(provider_id))
}

async fn provider_names(store: &ProviderStore, provider_ids: BTreeSet<String>) -> StorageResult<BTreeMap<String, String>> {
    if provider_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = providers::Entity::find()
        .filter(providers::Column::Id.is_in(provider_ids))
        .all(store.connection())
        .await?;
    Ok(records.into_iter().map(|record| (record.id, record.name)).collect())
}

async fn key_name(store: &ProviderStore, provider_id: &str, key_id: &str) -> StorageResult<String> {
    provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_api_keys::Column::Id.eq(key_id))
        .one(store.connection())
        .await?
        .map(|record| record.name)
        .ok_or_else(|| missing_key(provider_id, key_id))
}

async fn key_names(store: &ProviderStore, keys: &[provider_quick_import_keys::Model]) -> StorageResult<BTreeMap<(String, String), String>> {
    let provider_ids = keys.iter().map(|key| key.provider_id.clone()).collect::<BTreeSet<_>>();
    let key_ids = keys.iter().map(|key| key.key_id.clone()).collect::<BTreeSet<_>>();
    if provider_ids.is_empty() || key_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.is_in(provider_ids))
        .filter(provider_api_keys::Column::Id.is_in(key_ids))
        .all(store.connection())
        .await?;
    Ok(records.into_iter().map(|record| ((record.provider_id, record.id), record.name)).collect())
}

fn require_provider_name(names: &BTreeMap<String, String>, provider_id: &str) -> StorageResult<String> {
    names.get(provider_id).cloned().ok_or_else(|| missing_provider(provider_id))
}

fn require_key_name(names: &BTreeMap<(String, String), String>, provider_id: &str, key_id: &str) -> StorageResult<String> {
    names
        .get(&(provider_id.to_owned(), key_id.to_owned()))
        .cloned()
        .ok_or_else(|| missing_key(provider_id, key_id))
}

fn missing_provider(provider_id: &str) -> StorageError {
    StorageError::Database(format!("quick import sync source provider missing: {provider_id}"))
}

fn missing_key(provider_id: &str, key_id: &str) -> StorageError {
    StorageError::Database(format!("quick import sync local api key missing: {provider_id}/{key_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_provider_name_is_explicit() {
        let error = require_provider_name(&BTreeMap::new(), "provider-1").unwrap_err();
        assert_database_error(error, "quick import sync source provider missing: provider-1");
    }

    #[test]
    fn missing_key_name_is_explicit() {
        let error = require_key_name(&BTreeMap::new(), "provider-1", "key-1").unwrap_err();
        assert_database_error(error, "quick import sync local api key missing: provider-1/key-1");
    }

    fn assert_database_error(error: StorageError, expected: &str) {
        match error {
            StorageError::Database(message) => assert_eq!(message, expected),
            other => panic!("expected database error, got {other:?}"),
        }
    }
}
