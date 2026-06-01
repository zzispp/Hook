use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderModelRecordBatchUpdate, ProviderModelRecordInput, ProviderModelRecordPatch,
    record::provider_models,
    repository::ProviderStore,
    repository_helpers::{apply_provider_model_patch, provider_model_response},
};

pub async fn create_model_binding(store: &ProviderStore, input: ProviderModelRecordInput) -> StorageResult<types::provider::ProviderModelBinding> {
    let record = model_binding_active_model(store, input, time::OffsetDateTime::now_utc())?
        .insert(store.connection())
        .await?;
    provider_model_response(record)
}

pub async fn batch_update_model_bindings(
    store: &ProviderStore,
    input: ProviderModelRecordBatchUpdate,
) -> StorageResult<Vec<types::provider::ProviderModelBinding>> {
    let tx = store.connection().begin().await?;
    delete_model_bindings(&tx, &input.provider_id, input.delete_ids).await?;
    insert_model_bindings(store, &tx, input.create).await?;
    tx.commit().await?;
    model_bindings_for_provider(store, &input.provider_id).await
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

async fn delete_model_bindings<C>(connection: &C, provider_id: &str, ids: Vec<String>) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    if ids.is_empty() {
        return Ok(());
    }
    provider_models::Entity::delete_many()
        .filter(provider_models::Column::ProviderId.eq(provider_id))
        .filter(provider_models::Column::Id.is_in(ids))
        .exec(connection)
        .await?;
    Ok(())
}

async fn insert_model_bindings(store: &ProviderStore, connection: &sea_orm::DatabaseTransaction, inputs: Vec<ProviderModelRecordInput>) -> StorageResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = inputs
        .into_iter()
        .map(|input| model_binding_active_model(store, input, now))
        .collect::<StorageResult<Vec<_>>>()?;
    provider_models::Entity::insert_many(records).exec_without_returning(connection).await?;
    Ok(())
}

fn model_binding_active_model(
    store: &ProviderStore,
    input: ProviderModelRecordInput,
    now: time::OffsetDateTime,
) -> StorageResult<provider_models::ActiveModel> {
    Ok(provider_models::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(input.provider_id),
        global_model_id: Set(input.global_model_id),
        provider_model_name: Set(input.provider_model_name),
        provider_model_mappings: Set(json::encode_optional(&input.provider_model_mapping)?),
        is_active: Set(input.is_active),
        config: Set(json::encode_optional(&input.config)?),
        created_at: Set(now),
        updated_at: Set(now),
    })
}
