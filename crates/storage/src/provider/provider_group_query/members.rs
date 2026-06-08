use std::collections::BTreeMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::{StorageError, StorageResult};

use super::super::{
    ProviderStore,
    record::{ProviderGroupProviderRecord, ProviderKeyGroupKeyRecord, provider_group_providers, provider_key_group_keys},
};
use super::mapping::unique;

pub async fn provider_ids_for_groups(store: &ProviderStore, group_ids: &[String]) -> StorageResult<Vec<String>> {
    if group_ids.is_empty() {
        return Ok(Vec::new());
    }
    provider_group_providers::Entity::find()
        .filter(provider_group_providers::Column::ProviderGroupId.is_in(group_ids.iter().cloned()))
        .order_by_asc(provider_group_providers::Column::ProviderId)
        .all(store.connection())
        .await
        .map(|records| unique(records.into_iter().map(|record| record.provider_id)))
        .map_err(StorageError::from)
}

pub async fn provider_key_ids_for_key_groups(store: &ProviderStore, group_ids: &[String]) -> StorageResult<Vec<String>> {
    if group_ids.is_empty() {
        return Ok(Vec::new());
    }
    provider_key_group_keys::Entity::find()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.is_in(group_ids.iter().cloned()))
        .order_by_asc(provider_key_group_keys::Column::ProviderKeyId)
        .all(store.connection())
        .await
        .map(|records| unique(records.into_iter().map(|record| record.provider_key_id)))
        .map_err(StorageError::from)
}

pub async fn provider_ids_for_group(store: &ProviderStore, group_id: &str) -> StorageResult<Vec<String>> {
    provider_group_providers::Entity::find()
        .filter(provider_group_providers::Column::ProviderGroupId.eq(group_id))
        .order_by_asc(provider_group_providers::Column::ProviderId)
        .all(store.connection())
        .await
        .map(|records| records.into_iter().map(|record| record.provider_id).collect())
        .map_err(StorageError::from)
}

pub async fn provider_key_ids_for_group(store: &ProviderStore, group_id: &str) -> StorageResult<Vec<String>> {
    provider_key_group_keys::Entity::find()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.eq(group_id))
        .order_by_asc(provider_key_group_keys::Column::ProviderKeyId)
        .all(store.connection())
        .await
        .map(|records| records.into_iter().map(|record| record.provider_key_id).collect())
        .map_err(StorageError::from)
}

pub async fn provider_ids_by_group_ids(store: &ProviderStore, group_ids: &[String]) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if group_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_group_providers::Entity::find()
        .filter(provider_group_providers::Column::ProviderGroupId.is_in(group_ids.iter().cloned()))
        .order_by_asc(provider_group_providers::Column::ProviderId)
        .all(store.connection())
        .await?;
    Ok(provider_member_map(records))
}

pub async fn provider_key_ids_by_group_ids(store: &ProviderStore, group_ids: &[String]) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if group_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_key_group_keys::Entity::find()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.is_in(group_ids.iter().cloned()))
        .order_by_asc(provider_key_group_keys::Column::ProviderKeyId)
        .all(store.connection())
        .await?;
    Ok(provider_key_member_map(records))
}

pub async fn replace_provider_group_members(
    store: &ProviderStore,
    group_id: &str,
    provider_ids: Vec<String>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    provider_group_providers::Entity::delete_many()
        .filter(provider_group_providers::Column::ProviderGroupId.eq(group_id))
        .exec(tx)
        .await?;
    insert_provider_group_members(store, group_id, provider_ids, tx).await
}

pub async fn replace_provider_key_group_members(
    store: &ProviderStore,
    group_id: &str,
    key_ids: Vec<String>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    provider_key_group_keys::Entity::delete_many()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.eq(group_id))
        .exec(tx)
        .await?;
    insert_provider_key_group_members(store, group_id, key_ids, tx).await
}

async fn insert_provider_group_members(
    store: &ProviderStore,
    group_id: &str,
    provider_ids: Vec<String>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    if provider_ids.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = provider_ids
        .into_iter()
        .map(|provider_id| provider_group_member_active_model(store.next_id(), group_id, provider_id, now));
    provider_group_providers::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn insert_provider_key_group_members(
    store: &ProviderStore,
    group_id: &str,
    key_ids: Vec<String>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    if key_ids.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = key_ids
        .into_iter()
        .map(|provider_key_id| provider_key_group_member_active_model(store.next_id(), group_id, provider_key_id, now));
    provider_key_group_keys::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

fn provider_member_map(records: Vec<ProviderGroupProviderRecord>) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        map.entry(record.provider_group_id).or_default().push(record.provider_id);
    }
    map
}

fn provider_key_member_map(records: Vec<ProviderKeyGroupKeyRecord>) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        map.entry(record.provider_key_group_id).or_default().push(record.provider_key_id);
    }
    map
}

fn provider_group_member_active_model(id: String, group_id: &str, provider_id: String, now: time::OffsetDateTime) -> provider_group_providers::ActiveModel {
    provider_group_providers::ActiveModel {
        id: Set(id),
        provider_group_id: Set(group_id.to_owned()),
        provider_id: Set(provider_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn provider_key_group_member_active_model(
    id: String,
    group_id: &str,
    provider_key_id: String,
    now: time::OffsetDateTime,
) -> provider_key_group_keys::ActiveModel {
    provider_key_group_keys::ActiveModel {
        id: Set(id),
        provider_key_group_id: Set(group_id.to_owned()),
        provider_key_id: Set(provider_key_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}
