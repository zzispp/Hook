use async_trait::async_trait;
use serde_json::Value;
use types::model::{
    BatchDeleteGlobalModelsResponse, GlobalModelCreate, GlobalModelListRequest, GlobalModelListResponse, GlobalModelProvidersResponse, GlobalModelResponse,
    GlobalModelUpdate, GlobalModelWithStats, ModelCatalogResponse,
};

use super::ModelResult;

#[async_trait]
pub trait ModelRepository: Send + Sync + 'static {
    async fn create_global_model(&self, input: GlobalModelCreate) -> ModelResult<GlobalModelResponse>;
    async fn update_global_model(&self, id: &str, input: GlobalModelUpdate) -> ModelResult<GlobalModelResponse>;
    async fn delete_global_model(&self, id: &str) -> ModelResult<()>;
    async fn find_global_model_by_name(&self, name: &str) -> ModelResult<Option<GlobalModelResponse>>;
    async fn get_global_model(&self, id: &str) -> ModelResult<Option<GlobalModelWithStats>>;
    async fn list_global_models(&self, request: GlobalModelListRequest) -> ModelResult<GlobalModelListResponse>;
    async fn global_model_providers(&self, id: &str) -> ModelResult<GlobalModelProvidersResponse>;
    async fn catalog(&self) -> ModelResult<ModelCatalogResponse>;
}

#[async_trait]
pub trait ExternalModelCatalog: Send + Sync + 'static {
    async fn models_dev(&self) -> ModelResult<Value>;
}

#[async_trait]
pub trait ModelUseCase: Send + Sync + 'static {
    async fn create_global_model(&self, input: GlobalModelCreate) -> ModelResult<GlobalModelResponse>;
    async fn update_global_model(&self, id: &str, input: GlobalModelUpdate) -> ModelResult<GlobalModelResponse>;
    async fn delete_global_model(&self, id: &str) -> ModelResult<()>;
    async fn batch_delete_global_models(&self, ids: Vec<String>) -> ModelResult<BatchDeleteGlobalModelsResponse>;
    async fn get_global_model(&self, id: &str) -> ModelResult<GlobalModelWithStats>;
    async fn list_global_models(&self, request: GlobalModelListRequest) -> ModelResult<GlobalModelListResponse>;
    async fn global_model_providers(&self, id: &str) -> ModelResult<GlobalModelProvidersResponse>;
    async fn catalog(&self) -> ModelResult<ModelCatalogResponse>;
    async fn external_models(&self) -> ModelResult<Value>;
}
