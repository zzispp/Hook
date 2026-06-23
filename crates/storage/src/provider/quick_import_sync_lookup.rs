use std::collections::{BTreeMap, BTreeSet};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{StorageError, StorageResult};

use super::{
    ProviderStore,
    record::{provider_api_keys, provider_quick_import_keys, providers},
};

pub(super) async fn provider_name(store: &ProviderStore, provider_id: &str) -> StorageResult<String> {
    providers::Entity::find_by_id(provider_id.to_owned())
        .one(store.connection())
        .await?
        .map(|record| record.name)
        .ok_or_else(|| missing_provider(provider_id))
}

pub(super) async fn provider_names(store: &ProviderStore, provider_ids: BTreeSet<String>) -> StorageResult<BTreeMap<String, String>> {
    if provider_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = providers::Entity::find()
        .filter(providers::Column::Id.is_in(provider_ids))
        .all(store.connection())
        .await?;
    Ok(records.into_iter().map(|record| (record.id, record.name)).collect())
}

pub(super) async fn key_name(store: &ProviderStore, provider_id: &str, key_id: &str) -> StorageResult<String> {
    provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_api_keys::Column::Id.eq(key_id))
        .one(store.connection())
        .await?
        .map(|record| record.name)
        .ok_or_else(|| missing_key(provider_id, key_id))
}

pub(super) async fn key_names(store: &ProviderStore, keys: &[provider_quick_import_keys::Model]) -> StorageResult<BTreeMap<(String, String), String>> {
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

pub(super) fn require_provider_name(names: &BTreeMap<String, String>, provider_id: &str) -> StorageResult<String> {
    names.get(provider_id).cloned().ok_or_else(|| missing_provider(provider_id))
}

pub(super) fn require_key_name(names: &BTreeMap<(String, String), String>, provider_id: &str, key_id: &str) -> StorageResult<String> {
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
