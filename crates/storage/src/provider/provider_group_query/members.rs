use std::collections::BTreeMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use types::provider::{ProviderGroupMember, ProviderGroupMemberInput, ProviderKeyGroupMember, ProviderKeyGroupMemberInput};

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

pub async fn provider_priorities_for_groups(store: &ProviderStore, group_ids: &[String]) -> StorageResult<BTreeMap<String, i32>> {
    if group_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_group_providers::Entity::find()
        .filter(provider_group_providers::Column::ProviderGroupId.is_in(group_ids.iter().cloned()))
        .all(store.connection())
        .await?;
    Ok(min_provider_priority_map(records))
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

pub async fn provider_key_priorities_for_key_groups(store: &ProviderStore, group_ids: &[String]) -> StorageResult<BTreeMap<String, i32>> {
    if group_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_key_group_keys::Entity::find()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.is_in(group_ids.iter().cloned()))
        .all(store.connection())
        .await?;
    Ok(min_provider_key_priority_map(records))
}

pub async fn provider_members_for_group(store: &ProviderStore, group_id: &str) -> StorageResult<Vec<ProviderGroupMember>> {
    provider_group_providers::Entity::find()
        .filter(provider_group_providers::Column::ProviderGroupId.eq(group_id))
        .order_by_asc(provider_group_providers::Column::Priority)
        .order_by_asc(provider_group_providers::Column::ProviderId)
        .all(store.connection())
        .await
        .map(|records| records.into_iter().map(provider_member).collect())
        .map_err(StorageError::from)
}

pub async fn provider_key_members_for_group(store: &ProviderStore, group_id: &str) -> StorageResult<Vec<ProviderKeyGroupMember>> {
    provider_key_group_keys::Entity::find()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.eq(group_id))
        .order_by_asc(provider_key_group_keys::Column::Priority)
        .order_by_asc(provider_key_group_keys::Column::ProviderKeyId)
        .all(store.connection())
        .await
        .map(|records| records.into_iter().map(provider_key_member).collect())
        .map_err(StorageError::from)
}

pub async fn provider_members_by_group_ids(store: &ProviderStore, group_ids: &[String]) -> StorageResult<BTreeMap<String, Vec<ProviderGroupMember>>> {
    if group_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_group_providers::Entity::find()
        .filter(provider_group_providers::Column::ProviderGroupId.is_in(group_ids.iter().cloned()))
        .order_by_asc(provider_group_providers::Column::Priority)
        .order_by_asc(provider_group_providers::Column::ProviderId)
        .all(store.connection())
        .await?;
    Ok(provider_member_map(records))
}

pub async fn provider_key_members_by_group_ids(store: &ProviderStore, group_ids: &[String]) -> StorageResult<BTreeMap<String, Vec<ProviderKeyGroupMember>>> {
    if group_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_key_group_keys::Entity::find()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.is_in(group_ids.iter().cloned()))
        .order_by_asc(provider_key_group_keys::Column::Priority)
        .order_by_asc(provider_key_group_keys::Column::ProviderKeyId)
        .all(store.connection())
        .await?;
    Ok(provider_key_member_map(records))
}

pub async fn replace_provider_group_members(
    store: &ProviderStore,
    group_id: &str,
    provider_members: Vec<ProviderGroupMemberInput>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    provider_group_providers::Entity::delete_many()
        .filter(provider_group_providers::Column::ProviderGroupId.eq(group_id))
        .exec(tx)
        .await?;
    insert_provider_group_members(store, group_id, provider_members, tx).await
}

pub async fn replace_provider_key_group_members(
    store: &ProviderStore,
    group_id: &str,
    key_members: Vec<ProviderKeyGroupMemberInput>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    provider_key_group_keys::Entity::delete_many()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.eq(group_id))
        .exec(tx)
        .await?;
    insert_provider_key_group_members(store, group_id, key_members, tx).await
}

pub(in crate::provider) async fn insert_provider_group_members(
    store: &ProviderStore,
    group_id: &str,
    provider_members: Vec<ProviderGroupMemberInput>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    if provider_members.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = provider_members
        .into_iter()
        .map(|member| provider_group_member_active_model(store.next_id(), group_id, member, now));
    provider_group_providers::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn insert_provider_key_group_members(
    store: &ProviderStore,
    group_id: &str,
    key_members: Vec<ProviderKeyGroupMemberInput>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    if key_members.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = key_members
        .into_iter()
        .map(|member| provider_key_group_member_active_model(store.next_id(), group_id, member, now));
    provider_key_group_keys::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

fn provider_member_map(records: Vec<ProviderGroupProviderRecord>) -> BTreeMap<String, Vec<ProviderGroupMember>> {
    let mut map = BTreeMap::<String, Vec<ProviderGroupMember>>::new();
    for record in records {
        map.entry(record.provider_group_id.clone()).or_default().push(provider_member(record));
    }
    map
}

fn provider_key_member_map(records: Vec<ProviderKeyGroupKeyRecord>) -> BTreeMap<String, Vec<ProviderKeyGroupMember>> {
    let mut map = BTreeMap::<String, Vec<ProviderKeyGroupMember>>::new();
    for record in records {
        map.entry(record.provider_key_group_id.clone()).or_default().push(provider_key_member(record));
    }
    map
}

fn provider_member(record: ProviderGroupProviderRecord) -> ProviderGroupMember {
    ProviderGroupMember {
        provider_id: record.provider_id,
        priority: record.priority,
    }
}

fn provider_key_member(record: ProviderKeyGroupKeyRecord) -> ProviderKeyGroupMember {
    ProviderKeyGroupMember {
        provider_key_id: record.provider_key_id,
        priority: record.priority,
    }
}

fn min_provider_priority_map(records: Vec<ProviderGroupProviderRecord>) -> BTreeMap<String, i32> {
    let mut map = BTreeMap::new();
    for record in records {
        merge_min_priority(&mut map, record.provider_id, record.priority);
    }
    map
}

fn min_provider_key_priority_map(records: Vec<ProviderKeyGroupKeyRecord>) -> BTreeMap<String, i32> {
    let mut map = BTreeMap::new();
    for record in records {
        merge_min_priority(&mut map, record.provider_key_id, record.priority);
    }
    map
}

fn merge_min_priority(map: &mut BTreeMap<String, i32>, id: String, priority: i32) {
    map.entry(id).and_modify(|current| *current = (*current).min(priority)).or_insert(priority);
}

fn provider_group_member_active_model(
    id: String,
    group_id: &str,
    member: ProviderGroupMemberInput,
    now: time::OffsetDateTime,
) -> provider_group_providers::ActiveModel {
    provider_group_providers::ActiveModel {
        id: Set(id),
        provider_group_id: Set(group_id.to_owned()),
        provider_id: Set(member.provider_id),
        priority: Set(member.priority),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn provider_key_group_member_active_model(
    id: String,
    group_id: &str,
    member: ProviderKeyGroupMemberInput,
    now: time::OffsetDateTime,
) -> provider_key_group_keys::ActiveModel {
    provider_key_group_keys::ActiveModel {
        id: Set(id),
        provider_key_group_id: Set(group_id.to_owned()),
        provider_key_id: Set(member.provider_key_id),
        priority: Set(member.priority),
        created_at: Set(now),
        updated_at: Set(now),
    }
}
