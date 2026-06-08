use std::collections::BTreeMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::{StorageError, StorageResult};

use super::GroupStore;
use super::binding_maps::{
    binding_model_ids, binding_provider_group_ids, binding_provider_key_group_ids, binding_user_group_codes, model_bindings_by_group,
    provider_group_bindings_by_group, provider_key_group_bindings_by_group, user_group_bindings_by_group,
};
use crate::group::record::{billing_group_models, billing_group_provider_groups, billing_group_provider_key_groups, billing_group_user_groups};

pub async fn delete_group_bindings(group_code: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    billing_group_models::Entity::delete_many()
        .filter(billing_group_models::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    billing_group_provider_groups::Entity::delete_many()
        .filter(billing_group_provider_groups::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    billing_group_provider_key_groups::Entity::delete_many()
        .filter(billing_group_provider_key_groups::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    billing_group_user_groups::Entity::delete_many()
        .filter(billing_group_user_groups::Column::BillingGroupCode.eq(group_code))
        .exec(tx)
        .await?;
    Ok(())
}

pub async fn replace_group_models(group_code: &str, model_ids: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    billing_group_models::Entity::delete_many()
        .filter(billing_group_models::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    insert_group_models(group_code, model_ids, store, tx).await
}

pub async fn replace_group_provider_groups(
    group_code: &str,
    provider_group_ids: Vec<String>,
    store: &GroupStore,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    billing_group_provider_groups::Entity::delete_many()
        .filter(billing_group_provider_groups::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    insert_group_provider_groups(group_code, provider_group_ids, store, tx).await
}

pub async fn replace_group_provider_key_groups(
    group_code: &str,
    key_group_ids: Vec<String>,
    store: &GroupStore,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    billing_group_provider_key_groups::Entity::delete_many()
        .filter(billing_group_provider_key_groups::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    insert_group_provider_key_groups(group_code, key_group_ids, store, tx).await
}

pub async fn replace_group_user_groups(
    group_code: &str,
    user_group_codes: Vec<String>,
    store: &GroupStore,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    billing_group_user_groups::Entity::delete_many()
        .filter(billing_group_user_groups::Column::BillingGroupCode.eq(group_code))
        .exec(tx)
        .await?;
    insert_group_user_groups(group_code, user_group_codes, store, tx).await
}

pub async fn model_ids_for_group(group_code: &str, db: &sea_orm::DatabaseConnection) -> StorageResult<Vec<String>> {
    billing_group_models::Entity::find()
        .filter(billing_group_models::Column::GroupCode.eq(group_code))
        .order_by_asc(billing_group_models::Column::GlobalModelId)
        .all(db)
        .await
        .map(binding_model_ids)
        .map_err(StorageError::from)
}

pub async fn provider_group_ids_for_group(group_code: &str, db: &sea_orm::DatabaseConnection) -> StorageResult<Vec<String>> {
    billing_group_provider_groups::Entity::find()
        .filter(billing_group_provider_groups::Column::GroupCode.eq(group_code))
        .order_by_asc(billing_group_provider_groups::Column::ProviderGroupId)
        .all(db)
        .await
        .map(binding_provider_group_ids)
        .map_err(StorageError::from)
}

pub async fn provider_key_group_ids_for_group(group_code: &str, db: &sea_orm::DatabaseConnection) -> StorageResult<Vec<String>> {
    billing_group_provider_key_groups::Entity::find()
        .filter(billing_group_provider_key_groups::Column::GroupCode.eq(group_code))
        .order_by_asc(billing_group_provider_key_groups::Column::ProviderKeyGroupId)
        .all(db)
        .await
        .map(binding_provider_key_group_ids)
        .map_err(StorageError::from)
}

pub async fn user_group_codes_for_group(group_code: &str, db: &sea_orm::DatabaseConnection) -> StorageResult<Vec<String>> {
    billing_group_user_groups::Entity::find()
        .filter(billing_group_user_groups::Column::BillingGroupCode.eq(group_code))
        .order_by_asc(billing_group_user_groups::Column::UserGroupCode)
        .all(db)
        .await
        .map(binding_user_group_codes)
        .map_err(StorageError::from)
}

pub async fn model_ids_by_group_codes(codes: Vec<String>, db: &sea_orm::DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = billing_group_models::Entity::find()
        .filter(billing_group_models::Column::GroupCode.is_in(codes))
        .order_by_asc(billing_group_models::Column::GlobalModelId)
        .all(db)
        .await?;
    Ok(model_bindings_by_group(records))
}

pub async fn provider_group_ids_by_group_codes(codes: Vec<String>, db: &sea_orm::DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = billing_group_provider_groups::Entity::find()
        .filter(billing_group_provider_groups::Column::GroupCode.is_in(codes))
        .order_by_asc(billing_group_provider_groups::Column::ProviderGroupId)
        .all(db)
        .await?;
    Ok(provider_group_bindings_by_group(records))
}

pub async fn provider_key_group_ids_by_group_codes(codes: Vec<String>, db: &sea_orm::DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = billing_group_provider_key_groups::Entity::find()
        .filter(billing_group_provider_key_groups::Column::GroupCode.is_in(codes))
        .order_by_asc(billing_group_provider_key_groups::Column::ProviderKeyGroupId)
        .all(db)
        .await?;
    Ok(provider_key_group_bindings_by_group(records))
}

pub async fn user_group_codes_by_group_codes(codes: Vec<String>, db: &sea_orm::DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = billing_group_user_groups::Entity::find()
        .filter(billing_group_user_groups::Column::BillingGroupCode.is_in(codes))
        .order_by_asc(billing_group_user_groups::Column::UserGroupCode)
        .all(db)
        .await?;
    Ok(user_group_bindings_by_group(records))
}

async fn insert_group_models(group_code: &str, model_ids: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if model_ids.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = model_ids.into_iter().map(|model_id| billing_group_models::ActiveModel {
        id: Set(store.database.next_id()),
        group_code: Set(group_code.to_owned()),
        global_model_id: Set(model_id),
        created_at: Set(now),
        updated_at: Set(now),
    });
    billing_group_models::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn insert_group_provider_groups(
    group_code: &str,
    provider_group_ids: Vec<String>,
    store: &GroupStore,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    if provider_group_ids.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = provider_group_ids
        .into_iter()
        .map(|provider_group_id| provider_group_active_model(store, group_code, provider_group_id, now));
    billing_group_provider_groups::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn insert_group_provider_key_groups(
    group_code: &str,
    key_group_ids: Vec<String>,
    store: &GroupStore,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    if key_group_ids.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = key_group_ids
        .into_iter()
        .map(|provider_key_group_id| key_group_active_model(store, group_code, provider_key_group_id, now));
    billing_group_provider_key_groups::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn insert_group_user_groups(group_code: &str, user_group_codes: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if user_group_codes.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = user_group_codes
        .into_iter()
        .map(|user_group_code| user_group_active_model(store, group_code, user_group_code, now));
    billing_group_user_groups::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

fn provider_group_active_model(
    store: &GroupStore,
    group_code: &str,
    provider_group_id: String,
    now: time::OffsetDateTime,
) -> billing_group_provider_groups::ActiveModel {
    billing_group_provider_groups::ActiveModel {
        id: Set(store.database.next_id()),
        group_code: Set(group_code.to_owned()),
        provider_group_id: Set(provider_group_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn key_group_active_model(
    store: &GroupStore,
    group_code: &str,
    provider_key_group_id: String,
    now: time::OffsetDateTime,
) -> billing_group_provider_key_groups::ActiveModel {
    billing_group_provider_key_groups::ActiveModel {
        id: Set(store.database.next_id()),
        group_code: Set(group_code.to_owned()),
        provider_key_group_id: Set(provider_key_group_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn user_group_active_model(store: &GroupStore, group_code: &str, user_group_code: String, now: time::OffsetDateTime) -> billing_group_user_groups::ActiveModel {
    billing_group_user_groups::ActiveModel {
        id: Set(store.database.next_id()),
        billing_group_code: Set(group_code.to_owned()),
        user_group_code: Set(user_group_code),
        created_at: Set(now),
        updated_at: Set(now),
    }
}
