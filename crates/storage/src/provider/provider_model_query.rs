use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderModelRecordInput, ProviderModelRecordPatch,
    record::provider_models,
    repository::ProviderStore,
    repository_helpers::{apply_provider_model_patch, provider_model_response},
};

pub async fn create_model_binding(store: &ProviderStore, input: ProviderModelRecordInput) -> StorageResult<types::provider::ProviderModelBinding> {
    let now = time::OffsetDateTime::now_utc();
    let record = provider_models::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(input.provider_id),
        global_model_id: Set(input.global_model_id),
        provider_model_name: Set(input.provider_model_name),
        provider_model_mappings: Set(json::encode_optional(&input.provider_model_mapping)?),
        is_active: Set(input.is_active),
        price_per_request: Set(input.price_per_request),
        tiered_pricing: Set(json::encode_optional(&input.tiered_pricing)?),
        config: Set(json::encode_optional(&input.config)?),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(store.connection())
    .await?;
    provider_model_response(record)
}

pub async fn model_bindings_for_provider(store: &ProviderStore, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderModelBinding>> {
    let records = provider_models::Entity::find()
        .filter(provider_models::Column::ProviderId.eq(provider_id))
        .order_by_asc(provider_models::Column::ProviderModelName)
        .all(store.connection())
        .await?;
    records.into_iter().map(provider_model_response).collect()
}

pub async fn update_model_binding(
    store: &ProviderStore,
    provider_id: &str,
    model_id: &str,
    input: ProviderModelRecordPatch,
) -> StorageResult<types::provider::ProviderModelBinding> {
    let record = provider_model_record(store, provider_id, model_id).await?;
    let mut active: provider_models::ActiveModel = record.into();
    apply_provider_model_patch(&mut active, input)?;
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    let record = active.update(store.connection()).await?;
    provider_model_response(record)
}

pub async fn delete_model_binding(store: &ProviderStore, provider_id: &str, model_id: &str) -> StorageResult<()> {
    let record = provider_model_record(store, provider_id, model_id).await?;
    let active: provider_models::ActiveModel = record.into();
    active.delete(store.connection()).await?;
    Ok(())
}

async fn provider_model_record(store: &ProviderStore, provider_id: &str, model_id: &str) -> StorageResult<super::record::provider_models::Model> {
    let record = provider_models::Entity::find_by_id(model_id.to_owned())
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    if record.provider_id == provider_id {
        return Ok(record);
    }
    Err(StorageError::NotFound)
}
