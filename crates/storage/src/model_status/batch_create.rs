use types::model_status::{
    ModelStatusCheckBatchCreateFailure, ModelStatusCheckBatchCreateRequest, ModelStatusCheckBatchCreateResponse, ModelStatusCheckCreate,
};

use crate::{StorageResult, model_status::ModelStatusStore};

pub(super) async fn batch_create_checks(
    store: &ModelStatusStore,
    input: ModelStatusCheckBatchCreateRequest,
) -> StorageResult<ModelStatusCheckBatchCreateResponse> {
    let mut success_count = 0;
    let mut failed = Vec::new();
    for model_id in &input.global_model_ids {
        for api_format in &input.api_formats {
            match create_one(store, &input, model_id, api_format).await {
                Ok(()) => success_count += 1,
                Err(error) => failed.push(ModelStatusCheckBatchCreateFailure {
                    global_model_id: model_id.to_owned(),
                    api_format: api_format.to_owned(),
                    error: error.to_string(),
                }),
            }
        }
    }
    Ok(ModelStatusCheckBatchCreateResponse { success_count, failed })
}

async fn create_one(store: &ModelStatusStore, input: &ModelStatusCheckBatchCreateRequest, model_id: &str, api_format: &str) -> StorageResult<()> {
    let model = super::batch_update::model_label(store, model_id).await?;
    let payload = ModelStatusCheckCreate {
        name: format!("{} - {}", input.name_prefix.trim(), model),
        global_model_id: model_id.to_owned(),
        api_format: api_format.to_owned(),
        api_token_id: input.api_token_id.clone(),
        interval_seconds: input.interval_seconds,
        enabled: input.enabled,
    };
    store.create_check(payload).await?;
    Ok(())
}
