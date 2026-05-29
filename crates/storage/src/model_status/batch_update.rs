use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use types::model_status::{ModelStatusCheckBatchUpdateFailure, ModelStatusCheckBatchUpdateRequest, ModelStatusCheckBatchUpdateResponse};

use crate::{
    StorageError, StorageResult,
    model::global_models,
    model_status::{ModelStatusStore, entities::checks},
};

pub(super) async fn batch_update_checks(
    store: &ModelStatusStore,
    input: ModelStatusCheckBatchUpdateRequest,
) -> StorageResult<ModelStatusCheckBatchUpdateResponse> {
    let mut success_count = 0;
    let mut failed = Vec::new();
    for id in &input.ids {
        match update_one(store, id, &input).await {
            Ok(()) => success_count += 1,
            Err(error) => failed.push(ModelStatusCheckBatchUpdateFailure {
                id: id.to_owned(),
                error: error.to_string(),
            }),
        }
    }
    Ok(ModelStatusCheckBatchUpdateResponse { success_count, failed })
}

async fn update_one(store: &ModelStatusStore, id: &str, input: &ModelStatusCheckBatchUpdateRequest) -> StorageResult<()> {
    let record = checks::Entity::find_by_id(id.to_owned())
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: checks::ActiveModel = record.clone().into();
    apply_patch(store, &mut active, &record.global_model_id, input).await?;
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(store.connection()).await?;
    Ok(())
}

async fn apply_patch(
    store: &ModelStatusStore,
    active: &mut checks::ActiveModel,
    model_id: &str,
    input: &ModelStatusCheckBatchUpdateRequest,
) -> StorageResult<()> {
    if let Some(value) = input.enabled {
        active.enabled = Set(value);
    }
    if let Some(value) = input.interval_seconds {
        active.interval_seconds = Set(value);
    }
    if let Some(value) = input.api_token_id.as_deref() {
        active.api_token_id = Set(value.to_owned());
    }
    if let Some(prefix) = input.name_prefix.as_deref() {
        active.name = Set(format!("{} - {}", prefix.trim(), model_label(store, model_id).await?));
    }
    Ok(())
}

pub(super) async fn model_label(store: &ModelStatusStore, model_id: &str) -> StorageResult<String> {
    let record = global_models::Entity::find_by_id(model_id.to_owned())
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    let display_name = record.display_name.trim();
    if display_name.is_empty() {
        return Ok(record.name);
    }
    Ok(display_name.to_owned())
}
