use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::{
    StorageResult,
    api_token::api_token_records,
    group::billing_group_models,
    json,
    model_status::entities::checks as model_status_checks,
    provider::record::provider_api_keys,
    user::{UserActiveModel, UserEntity},
};

use super::provider_models;

pub(super) async fn delete_model_bindings<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    provider_models::Entity::delete_many()
        .filter(provider_models::Column::GlobalModelId.eq(model_id))
        .exec(connection)
        .await?;
    billing_group_models::Entity::delete_many()
        .filter(billing_group_models::Column::GlobalModelId.eq(model_id))
        .exec(connection)
        .await?;
    Ok(())
}

pub(super) async fn delete_model_status_checks<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    model_status_checks::Entity::delete_many()
        .filter(model_status_checks::Column::GlobalModelId.eq(model_id))
        .exec(connection)
        .await?;
    Ok(())
}

pub(super) async fn prune_model_references<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    prune_provider_api_key_model_ids(connection, model_id).await?;
    prune_api_token_model_ids(connection, model_id).await?;
    prune_user_model_ids(connection, model_id).await?;
    Ok(())
}

pub(super) async fn prune_provider_api_key_model_ids<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let records = provider_api_keys::Entity::find().all(connection).await?;
    let now = time::OffsetDateTime::now_utc();
    for record in records {
        let allowed_model_ids: Vec<String> = json::decode_required(record.allowed_model_ids.clone())?;
        let pruned_model_ids = remove_model_id(allowed_model_ids.clone(), model_id);
        if pruned_model_ids.len() == allowed_model_ids.len() {
            continue;
        }
        let mut active: provider_api_keys::ActiveModel = record.into();
        active.allowed_model_ids = Set(json::encode_required(&pruned_model_ids)?);
        active.updated_at = Set(now);
        active.update(connection).await?;
    }
    Ok(())
}

pub(super) async fn prune_api_token_model_ids<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let records = api_token_records::Entity::find().all(connection).await?;
    let now = time::OffsetDateTime::now_utc();
    for record in records {
        let allowed_model_ids: Vec<String> = json::decode_required(record.allowed_model_ids.clone())?;
        let pruned_model_ids = remove_model_id(allowed_model_ids.clone(), model_id);
        if pruned_model_ids.len() == allowed_model_ids.len() {
            continue;
        }
        let mut active: api_token_records::ActiveModel = record.into();
        active.allowed_model_ids = Set(json::encode_required(&pruned_model_ids)?);
        active.updated_at = Set(now);
        active.update(connection).await?;
    }
    Ok(())
}

pub(super) async fn prune_user_model_ids<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let records = UserEntity::find().all(connection).await?;
    let now = time::OffsetDateTime::now_utc();
    for record in records {
        let allowed_model_ids: Vec<String> = json::decode_required(record.allowed_model_ids.clone())?;
        let pruned_model_ids = remove_model_id(allowed_model_ids.clone(), model_id);
        if pruned_model_ids.len() == allowed_model_ids.len() {
            continue;
        }
        let mut active: UserActiveModel = record.into();
        active.allowed_model_ids = Set(json::encode_required(&pruned_model_ids)?);
        active.updated_at = Set(now);
        active.update(connection).await?;
    }
    Ok(())
}

pub(super) fn remove_model_id(model_ids: Vec<String>, model_id: &str) -> Vec<String> {
    model_ids.into_iter().filter(|value| value != model_id).collect()
}
