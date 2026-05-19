use async_trait::async_trait;
use model::application::{ModelError, ModelRepository, ModelResult};
use types::model::{
    GlobalModelCreate, GlobalModelListRequest, GlobalModelListResponse, GlobalModelProvidersResponse, GlobalModelResponse, GlobalModelUpdate,
    GlobalModelWithStats, ModelCatalogResponse,
};

use super::cache::ProxyCacheInvalidator;

#[derive(Clone)]
pub struct CachedModelRepository<R, C> {
    inner: R,
    cache: C,
}

impl<R, C> CachedModelRepository<R, C> {
    pub const fn new(inner: R, cache: C) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl<R, C> ModelRepository for CachedModelRepository<R, C>
where
    R: ModelRepository,
    C: ProxyCacheInvalidator,
{
    async fn create_global_model(&self, input: GlobalModelCreate) -> ModelResult<GlobalModelResponse> {
        let model = self.inner.create_global_model(input).await?;
        self.refresh_scheduling().await?;
        Ok(model)
    }

    async fn update_global_model(&self, id: &str, input: GlobalModelUpdate) -> ModelResult<GlobalModelResponse> {
        let model = self.inner.update_global_model(id, input).await?;
        self.refresh_scheduling().await?;
        Ok(model)
    }

    async fn delete_global_model(&self, id: &str) -> ModelResult<()> {
        self.inner.delete_global_model(id).await?;
        self.refresh_scheduling().await
    }

    async fn find_global_model_by_name(&self, name: &str) -> ModelResult<Option<GlobalModelResponse>> {
        self.inner.find_global_model_by_name(name).await
    }

    async fn get_global_model(&self, id: &str) -> ModelResult<Option<GlobalModelWithStats>> {
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
}

impl<R, C> CachedModelRepository<R, C>
where
    C: ProxyCacheInvalidator,
{
    async fn refresh_scheduling(&self) -> ModelResult<()> {
        self.cache.refresh_scheduling().await.map_err(cache_error)
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> ModelError {
    ModelError::Infrastructure(error.to_string())
}
