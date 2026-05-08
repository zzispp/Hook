use async_trait::async_trait;
use storage::{
    Database, StorageError,
    model::{GlobalModelRecordInput, GlobalModelRecordPatch, ModelStore},
};
use types::model::{
    GlobalModelCreate, GlobalModelListRequest, GlobalModelListResponse, GlobalModelProvidersResponse, GlobalModelResponse, GlobalModelUpdate,
    GlobalModelWithStats, ModelCatalogResponse, PatchField,
};

use crate::application::{ModelError, ModelRepository, ModelResult};

#[derive(Clone)]
pub struct StorageModelRepository {
    store: ModelStore,
}

impl StorageModelRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: ModelStore::new(database),
        }
    }
}

#[async_trait]
impl ModelRepository for StorageModelRepository {
    async fn create_global_model(&self, input: GlobalModelCreate) -> ModelResult<GlobalModelResponse> {
        self.store.create_global_model(record_input(input)).await.map_err(storage_error)
    }

    async fn update_global_model(&self, id: &str, input: GlobalModelUpdate) -> ModelResult<GlobalModelResponse> {
        self.store.update_global_model(id, record_patch(input)).await.map_err(storage_error)
    }

    async fn delete_global_model(&self, id: &str) -> ModelResult<()> {
        self.store.delete_global_model(id).await.map_err(storage_error)
    }

    async fn find_global_model_by_name(&self, name: &str) -> ModelResult<Option<GlobalModelResponse>> {
        self.store.find_global_model_by_name(name).await.map_err(storage_error)
    }

    async fn get_global_model(&self, id: &str) -> ModelResult<Option<GlobalModelWithStats>> {
        self.store.get_global_model_with_stats(id).await.map_err(storage_error)
    }

    async fn list_global_models(&self, request: GlobalModelListRequest) -> ModelResult<GlobalModelListResponse> {
        self.store.list_global_models(request).await.map_err(storage_error)
    }

    async fn global_model_providers(&self, id: &str) -> ModelResult<GlobalModelProvidersResponse> {
        self.store.global_model_providers(id).await.map_err(storage_error)
    }

    async fn catalog(&self) -> ModelResult<ModelCatalogResponse> {
        self.store.catalog().await.map_err(storage_error)
    }
}

fn record_input(input: GlobalModelCreate) -> GlobalModelRecordInput {
    GlobalModelRecordInput {
        name: input.name,
        display_name: input.display_name,
        default_price_per_request: input.default_price_per_request,
        default_tiered_pricing: input.default_tiered_pricing,
        supported_capabilities: input.supported_capabilities,
        config: input.config,
        is_active: input.is_active.unwrap_or(true),
    }
}

fn record_patch(input: GlobalModelUpdate) -> GlobalModelRecordPatch {
    GlobalModelRecordPatch {
        display_name: input.display_name,
        is_active: input.is_active,
        default_price_per_request: input.default_price_per_request,
        default_tiered_pricing: tiered_pricing_patch(input.default_tiered_pricing),
        supported_capabilities: input.supported_capabilities,
        config: input.config,
    }
}

fn tiered_pricing_patch(patch: PatchField<types::model::TieredPricingConfig>) -> Option<types::model::TieredPricingConfig> {
    match patch {
        PatchField::Value(value) => Some(value),
        PatchField::Missing | PatchField::Null => None,
    }
}

fn storage_error(error: StorageError) -> ModelError {
    match error {
        StorageError::NotFound => ModelError::NotFound,
        StorageError::Conflict(message) => ModelError::Conflict(message),
        StorageError::Database(message) => ModelError::Infrastructure(message),
    }
}
