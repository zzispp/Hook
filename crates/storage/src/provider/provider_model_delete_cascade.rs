use std::collections::BTreeSet;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use types::provider::ProviderQuickImportSyncStatus;

use crate::{StorageError, StorageResult, json};

use super::{
    record::{
        provider_api_keys, provider_key_model_mappings, provider_model_costs, provider_models, provider_quick_import_keys, provider_quick_import_sync_events,
    },
    repository::ProviderStore,
};

pub(super) async fn cascade_model_binding_delete(
    store: &ProviderStore,
    tx: &sea_orm::DatabaseTransaction,
    record: &provider_models::Model,
) -> StorageResult<()> {
    delete_model_costs(tx, &record.provider_id, &record.id).await?;
    let affected_keys = delete_key_model_mappings(tx, &record.provider_id, &record.id).await?;
    remove_allowed_model_from_keys(tx, &record.provider_id, &record.id).await?;
    mark_empty_quick_import_keys(store, tx, &record.provider_id, affected_keys).await
}

async fn delete_model_costs(tx: &sea_orm::DatabaseTransaction, provider_id: &str, model_id: &str) -> StorageResult<()> {
    provider_model_costs::Entity::delete_many()
        .filter(provider_model_costs::Column::ProviderId.eq(provider_id))
        .filter(provider_model_costs::Column::ProviderModelId.eq(model_id))
        .exec(tx)
        .await?;
    Ok(())
}

async fn delete_key_model_mappings(tx: &sea_orm::DatabaseTransaction, provider_id: &str, provider_model_id: &str) -> StorageResult<BTreeSet<String>> {
    let mappings = provider_key_model_mappings::Entity::find()
        .filter(provider_key_model_mappings::Column::ProviderId.eq(provider_id))
        .filter(provider_key_model_mappings::Column::ProviderModelId.eq(provider_model_id))
        .all(tx)
        .await?;
    let affected = mappings.iter().map(|mapping| mapping.key_id.clone()).collect();
    provider_key_model_mappings::Entity::delete_many()
        .filter(provider_key_model_mappings::Column::ProviderId.eq(provider_id))
        .filter(provider_key_model_mappings::Column::ProviderModelId.eq(provider_model_id))
        .exec(tx)
        .await?;
    Ok(affected)
}

async fn remove_allowed_model_from_keys(tx: &sea_orm::DatabaseTransaction, provider_id: &str, model_id: &str) -> StorageResult<()> {
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .all(tx)
        .await?;
    for record in records {
        update_allowed_models(tx, record, model_id).await?;
    }
    Ok(())
}

async fn update_allowed_models(tx: &sea_orm::DatabaseTransaction, record: provider_api_keys::Model, model_id: &str) -> StorageResult<()> {
    let mut allowed_model_ids = json::decode_required::<Vec<String>>(record.allowed_model_ids.clone())?;
    let before = allowed_model_ids.len();
    allowed_model_ids.retain(|id| id != model_id);
    if allowed_model_ids.len() == before {
        return Ok(());
    }
    let mut active: provider_api_keys::ActiveModel = record.into();
    active.allowed_model_ids = Set(json::encode_required(&allowed_model_ids)?);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(tx).await?;
    Ok(())
}

async fn mark_empty_quick_import_keys(
    store: &ProviderStore,
    tx: &sea_orm::DatabaseTransaction,
    provider_id: &str,
    key_ids: BTreeSet<String>,
) -> StorageResult<()> {
    for key_id in key_ids {
        if quick_import_key_has_models(tx, provider_id, &key_id).await? {
            continue;
        }
        disable_empty_quick_import_key(store, tx, provider_id, &key_id).await?;
    }
    Ok(())
}

async fn quick_import_key_has_models(tx: &sea_orm::DatabaseTransaction, provider_id: &str, key_id: &str) -> StorageResult<bool> {
    let mapping = provider_key_model_mappings::Entity::find()
        .filter(provider_key_model_mappings::Column::ProviderId.eq(provider_id))
        .filter(provider_key_model_mappings::Column::KeyId.eq(key_id))
        .one(tx)
        .await?;
    Ok(mapping.is_some())
}

async fn disable_empty_quick_import_key(store: &ProviderStore, tx: &sea_orm::DatabaseTransaction, provider_id: &str, key_id: &str) -> StorageResult<()> {
    let key = quick_import_key_record(tx, provider_id, key_id).await?;
    disable_api_key(tx, provider_id, key_id).await?;
    let old_statuses = json::decode_required::<Vec<ProviderQuickImportSyncStatus>>(key.sync_statuses.clone())?;
    update_quick_import_key_status(tx, key.clone()).await?;
    if !old_statuses.contains(&ProviderQuickImportSyncStatus::NoAssociatedModels) {
        insert_no_associated_models_event(store, tx, key).await?;
    }
    Ok(())
}

async fn quick_import_key_record(tx: &sea_orm::DatabaseTransaction, provider_id: &str, key_id: &str) -> StorageResult<provider_quick_import_keys::Model> {
    provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_quick_import_keys::Column::KeyId.eq(key_id))
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)
}

async fn disable_api_key(tx: &sea_orm::DatabaseTransaction, provider_id: &str, key_id: &str) -> StorageResult<()> {
    let record = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_api_keys::Column::Id.eq(key_id))
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: provider_api_keys::ActiveModel = record.into();
    active.is_active = Set(false);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(tx).await?;
    Ok(())
}

async fn update_quick_import_key_status(tx: &sea_orm::DatabaseTransaction, key: provider_quick_import_keys::Model) -> StorageResult<()> {
    let now = time::OffsetDateTime::now_utc();
    let mut active: provider_quick_import_keys::ActiveModel = key.into();
    active.sync_statuses = Set(json::encode_required(&vec![ProviderQuickImportSyncStatus::NoAssociatedModels])?);
    active.last_sync_error = Set(None);
    active.last_synced_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(tx).await?;
    Ok(())
}

async fn insert_no_associated_models_event(
    store: &ProviderStore,
    tx: &sea_orm::DatabaseTransaction,
    key: provider_quick_import_keys::Model,
) -> StorageResult<()> {
    provider_quick_import_sync_events::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(key.provider_id),
        source_id: Set(key.source_id),
        key_id: Set(Some(key.key_id)),
        status: Set(ProviderQuickImportSyncStatus::NoAssociatedModels.as_str().to_owned()),
        title: Set(format!("快捷导入同步异常：{}({}) 没有关联模型", key.upstream_token_name, key.upstream_token_id)),
        detail: Set("管理员删除了该密钥最后一个关联模型，系统已禁用本地密钥。".into()),
        payload_json: Set(None),
        created_at: Set(time::OffsetDateTime::now_utc()),
    }
    .insert(tx)
    .await?;
    Ok(())
}
