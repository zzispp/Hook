use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ExprTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use types::model::PatchField;
use types::provider::{ProviderGroupListRequest, ProviderKeyGroup, ProviderKeyGroupListResponse};

use crate::{StorageError, StorageResult};

use super::super::{
    ProviderKeyGroupRecordInput, ProviderKeyGroupRecordPatch, ProviderStore,
    record::{ProviderKeyGroupRecord, provider_key_groups},
};
use super::mapping::{apply_provider_key_group_patch, provider_key_group_active_model, provider_key_group_response};
use super::members::{provider_key_members_by_group_ids, provider_key_members_for_group, replace_provider_key_group_members};

pub async fn create_provider_key_group(store: &ProviderStore, input: ProviderKeyGroupRecordInput) -> StorageResult<ProviderKeyGroup> {
    let provider_key_members = input.provider_key_members.clone();
    let tx = store.connection().begin().await?;
    let record = provider_key_group_active_model(store.next_id(), input).insert(&tx).await?;
    replace_provider_key_group_members(store, &record.id, provider_key_members, &tx).await?;
    tx.commit().await?;
    find_provider_key_group(store, &record.id).await?.ok_or(StorageError::NotFound)
}

pub async fn update_provider_key_group(store: &ProviderStore, id: &str, input: ProviderKeyGroupRecordPatch) -> StorageResult<ProviderKeyGroup> {
    let record = find_provider_key_group_record(store, id).await?.ok_or(StorageError::NotFound)?;
    let key_patch = input.provider_key_members.clone();
    let tx = store.connection().begin().await?;
    let mut active: provider_key_groups::ActiveModel = record.into();
    apply_provider_key_group_patch(&mut active, input);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    let updated = active.update(&tx).await?;
    if let PatchField::Value(provider_key_members) = key_patch {
        replace_provider_key_group_members(store, &updated.id, provider_key_members, &tx).await?;
    }
    tx.commit().await?;
    find_provider_key_group(store, id).await?.ok_or(StorageError::NotFound)
}

pub async fn delete_provider_key_group(store: &ProviderStore, id: &str) -> StorageResult<()> {
    let record = find_provider_key_group_record(store, id).await?.ok_or(StorageError::NotFound)?;
    let active: provider_key_groups::ActiveModel = record.into();
    active.delete(store.connection()).await?;
    Ok(())
}

pub async fn find_provider_key_group(store: &ProviderStore, id_or_name: &str) -> StorageResult<Option<ProviderKeyGroup>> {
    match find_provider_key_group_record(store, id_or_name).await? {
        Some(record) => provider_key_group_from_record(store, record).await.map(Some),
        None => Ok(None),
    }
}

pub async fn list_provider_key_groups(store: &ProviderStore, request: ProviderGroupListRequest) -> StorageResult<ProviderKeyGroupListResponse> {
    let records = filtered_provider_key_groups(store, request.clone()).await?;
    let total = records.len() as u64;
    let page = records.into_iter().skip(request.skip as usize).take(request.limit as usize).collect();
    let groups = provider_key_groups_from_records(store, page).await?;
    Ok(ProviderKeyGroupListResponse { groups, total })
}

async fn filtered_provider_key_groups(store: &ProviderStore, request: ProviderGroupListRequest) -> StorageResult<Vec<ProviderKeyGroupRecord>> {
    let mut query = provider_key_groups::Entity::find();
    if let Some(search) = request.search.filter(|value| !value.is_empty()) {
        query = query.filter(
            provider_key_groups::Column::Name
                .contains(&search)
                .or(provider_key_groups::Column::Description.contains(&search)),
        );
    }
    query
        .order_by_asc(provider_key_groups::Column::SortOrder)
        .order_by_asc(provider_key_groups::Column::Name)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn find_provider_key_group_record(store: &ProviderStore, id_or_name: &str) -> StorageResult<Option<ProviderKeyGroupRecord>> {
    match provider_key_groups::Entity::find_by_id(id_or_name.to_owned()).one(store.connection()).await? {
        Some(record) => Ok(Some(record)),
        None => provider_key_groups::Entity::find()
            .filter(provider_key_groups::Column::Name.eq(id_or_name))
            .one(store.connection())
            .await
            .map_err(StorageError::from),
    }
}

async fn provider_key_group_from_record(store: &ProviderStore, record: ProviderKeyGroupRecord) -> StorageResult<ProviderKeyGroup> {
    let provider_key_members = provider_key_members_for_group(store, &record.id).await?;
    Ok(provider_key_group_response(record, provider_key_members))
}

async fn provider_key_groups_from_records(store: &ProviderStore, records: Vec<ProviderKeyGroupRecord>) -> StorageResult<Vec<ProviderKeyGroup>> {
    let ids = records.iter().map(|record| record.id.clone()).collect::<Vec<_>>();
    let members = provider_key_members_by_group_ids(store, &ids).await?;
    Ok(records
        .into_iter()
        .map(|record| provider_key_group_response(record.clone(), members.get(&record.id).cloned().unwrap_or_default()))
        .collect())
}
