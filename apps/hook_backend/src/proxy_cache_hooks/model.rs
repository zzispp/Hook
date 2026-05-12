use std::sync::Arc;

use async_trait::async_trait;
use model::application::{ModelError, ModelResult, ModelUseCase};
use serde_json::Value;
use types::model::{
    BatchDeleteGlobalModelsResponse, GlobalModelCreate, GlobalModelListRequest, GlobalModelListResponse, GlobalModelProvidersResponse, GlobalModelResponse,
    GlobalModelUpdate, GlobalModelWithStats, ModelCatalogResponse,
};

use crate::llm_proxy::LlmProxyCache;

pub struct ProxyCachedModelUseCase {
    inner: Arc<dyn ModelUseCase>,
    cache: LlmProxyCache,
}

impl ProxyCachedModelUseCase {
    pub fn new(inner: Arc<dyn ModelUseCase>, cache: LlmProxyCache) -> Self {
        Self { inner, cache }
    }

    async fn refresh_scheduling(&self) -> ModelResult<()> {
        self.cache.refresh_scheduling_snapshot().await.map(|_| ()).map_err(cache_error)
    }
}

#[async_trait]
impl ModelUseCase for ProxyCachedModelUseCase {
    async fn create_global_model(&self, input: GlobalModelCreate) -> ModelResult<GlobalModelResponse> {
        let value = self.inner.create_global_model(input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn update_global_model(&self, id: &str, input: GlobalModelUpdate) -> ModelResult<GlobalModelResponse> {
        let value = self.inner.update_global_model(id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn delete_global_model(&self, id: &str) -> ModelResult<()> {
        self.inner.delete_global_model(id).await?;
        self.refresh_scheduling().await
    }

    async fn batch_delete_global_models(&self, ids: Vec<String>) -> ModelResult<BatchDeleteGlobalModelsResponse> {
        let value = self.inner.batch_delete_global_models(ids).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn get_global_model(&self, id: &str) -> ModelResult<GlobalModelWithStats> {
        self.inner.get_global_model(id).await
    }

    async fn list_global_models(&self, request: GlobalModelListRequest) -> ModelResult<GlobalModelListResponse> {
        self.inner.list_global_models(request).await
    }

    async fn global_model_providers(&self, id: &str) -> ModelResult<GlobalModelProvidersResponse> {
        self.inner.global_model_providers(id).await
    }

    async fn catalog(&self) -> ModelResult<ModelCatalogResponse> {
        self.inner.catalog().await
    }

    async fn external_models(&self) -> ModelResult<Value> {
        self.inner.external_models().await
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> ModelError {
    ModelError::Infrastructure(error.to_string())
}
