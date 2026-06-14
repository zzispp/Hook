use async_trait::async_trait;
use types::model::{
    BatchDeleteFailure, BatchDeleteGlobalModelsResponse, GlobalModelCreate, GlobalModelListRequest, GlobalModelListResponse, GlobalModelProvidersResponse,
    GlobalModelResponse, GlobalModelUpdate, GlobalModelWithStats, ModelCatalogResponse,
};

use crate::application::{ExternalModelCatalog, ModelError, ModelRepository, ModelResult, ModelUseCase};

use super::validation::{sanitize_create, sanitize_update, validate_batch_delete, validate_create, validate_list_request, validate_update};

pub struct ModelService<R, E> {
    repository: R,
    external_catalog: E,
}

impl<R, E> ModelService<R, E>
where
    R: ModelRepository,
    E: ExternalModelCatalog,
{
    pub const fn new(repository: R, external_catalog: E) -> Self {
        Self { repository, external_catalog }
    }
}

#[async_trait]
impl<R, E> ModelUseCase for ModelService<R, E>
where
    R: ModelRepository,
    E: ExternalModelCatalog,
{
    async fn create_global_model(&self, input: GlobalModelCreate) -> ModelResult<GlobalModelResponse> {
        let input = sanitize_create(input);
        validate_create(&input)?;
        reject_duplicate_name(&self.repository, &input.name).await?;
        self.repository.create_global_model(input).await
    }

    async fn update_global_model(&self, id: &str, input: GlobalModelUpdate) -> ModelResult<GlobalModelResponse> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        self.repository.update_global_model(id, input).await
    }

    async fn delete_global_model(&self, id: &str) -> ModelResult<()> {
        self.repository.delete_global_model(id).await
    }

    async fn batch_delete_global_models(&self, ids: Vec<String>) -> ModelResult<BatchDeleteGlobalModelsResponse> {
        validate_batch_delete(&ids)?;
        let mut success_count = 0;
        let mut failed = Vec::new();
        for id in ids {
            match self.repository.delete_global_model(&id).await {
                Ok(()) => success_count += 1,
                Err(error) => failed.push(BatchDeleteFailure { id, error: error.to_string() }),
            }
        }
        Ok(BatchDeleteGlobalModelsResponse { success_count, failed })
    }

    async fn get_global_model(&self, id: &str) -> ModelResult<GlobalModelWithStats> {
        self.repository.get_global_model(id).await?.ok_or(ModelError::NotFound)
    }

    async fn list_global_models(&self, request: GlobalModelListRequest) -> ModelResult<GlobalModelListResponse> {
        validate_list_request(&request)?;
        self.repository.list_global_models(request).await
    }

    async fn list_user_global_models(&self, user_id: &str, request: GlobalModelListRequest) -> ModelResult<GlobalModelListResponse> {
        validate_list_request(&request)?;
        self.repository.list_user_global_models(user_id, request).await
    }

    async fn global_model_providers(&self, id: &str) -> ModelResult<GlobalModelProvidersResponse> {
        self.repository.global_model_providers(id).await
    }

    async fn catalog(&self) -> ModelResult<ModelCatalogResponse> {
        self.repository.catalog().await
    }

    async fn external_models(&self) -> ModelResult<serde_json::Value> {
        self.external_catalog.models_dev().await
    }
}

async fn reject_duplicate_name<R>(repository: &R, name: &str) -> ModelResult<()>
where
    R: ModelRepository,
{
    if repository.find_global_model_by_name(name).await?.is_some() {
        return Err(ModelError::Conflict(format!("GlobalModel with name '{name}' already exists")));
    }
    Ok(())
}
