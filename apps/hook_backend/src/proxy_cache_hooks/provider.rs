use std::sync::Arc;

use async_trait::async_trait;
use provider::application::{ProviderError, ProviderResult, ProviderUseCase};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCreate,
    ProviderEndpoint, ProviderEndpointCreate, ProviderEndpointUpdate, ProviderListRequest, ProviderListResponse, ProviderModelBinding,
    ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderUpdate, ProviderUpstreamModelsResponse, RequestRecordDetail,
    RequestRecordListRequest, RequestRecordListResponse,
};

use crate::llm_proxy::LlmProxyCache;

pub struct ProxyCachedProviderUseCase {
    inner: Arc<dyn ProviderUseCase>,
    cache: LlmProxyCache,
}

impl ProxyCachedProviderUseCase {
    pub fn new(inner: Arc<dyn ProviderUseCase>, cache: LlmProxyCache) -> Self {
        Self { inner, cache }
    }

    async fn refresh_scheduling(&self) -> ProviderResult<()> {
        self.cache.refresh_scheduling_snapshot().await.map(|_| ()).map_err(cache_error)
    }
}

#[async_trait]
impl ProviderUseCase for ProxyCachedProviderUseCase {
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider> {
        let value = self.inner.create_provider(input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider> {
        let value = self.inner.update_provider(id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn delete_provider(&self, id: &str) -> ProviderResult<()> {
        self.inner.delete_provider(id).await?;
        self.refresh_scheduling().await
    }

    async fn get_provider(&self, id: &str) -> ProviderResult<Provider> {
        self.inner.get_provider(id).await
    }

    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse> {
        self.inner.list_providers(request).await
    }

    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint> {
        let value = self.inner.create_endpoint(provider_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint> {
        let value = self.inner.update_endpoint(provider_id, endpoint_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()> {
        self.inner.delete_endpoint(provider_id, endpoint_id).await?;
        self.refresh_scheduling().await
    }

    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>> {
        self.inner.list_endpoints(provider_id).await
    }

    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate) -> ProviderResult<ProviderApiKey> {
        let value = self.inner.create_api_key(provider_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>> {
        self.inner.list_api_keys(provider_id).await
    }

    async fn fetch_upstream_models(&self, provider_id: &str) -> ProviderResult<ProviderUpstreamModelsResponse> {
        self.inner.fetch_upstream_models(provider_id).await
    }

    async fn update_api_key(&self, provider_id: &str, key_id: &str, input: ProviderApiKeyUpdate) -> ProviderResult<ProviderApiKey> {
        let value = self.inner.update_api_key(provider_id, key_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()> {
        self.inner.delete_api_key(provider_id, key_id).await?;
        self.refresh_scheduling().await
    }

    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding> {
        let value = self.inner.create_model_binding(provider_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>> {
        self.inner.list_model_bindings(provider_id).await
    }

    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding> {
        let value = self.inner.update_model_binding(provider_id, model_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()> {
        self.inner.delete_model_binding(provider_id, model_id).await?;
        self.refresh_scheduling().await
    }

    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse> {
        self.inner.list_request_records(request).await
    }

    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse> {
        self.inner.list_active_request_records(request).await
    }

    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail> {
        self.inner.get_request_record(request_id).await
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> ProviderError {
    ProviderError::Infrastructure(error.to_string())
}
