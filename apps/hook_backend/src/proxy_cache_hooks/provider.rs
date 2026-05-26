use async_trait::async_trait;
use provider::application::{ProviderApiKeySecret, ProviderError, ProviderRepository, ProviderResult};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCooldown,
    ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint, ProviderEndpointCreate, ProviderEndpointUpdate,
    ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderModelCostBatchUpsert,
    ProviderModelCostListResponse, ProviderUpdate, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse, UsageRecordListResponse,
};

use super::cache::{ProxyCacheInvalidator, combine_cache_results};

#[derive(Clone)]
pub struct CachedProviderRepository<R, C> {
    inner: R,
    cache: C,
}

impl<R, C> CachedProviderRepository<R, C> {
    pub const fn new(inner: R, cache: C) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl<R, C> ProviderRepository for CachedProviderRepository<R, C>
where
    R: ProviderRepository,
    C: ProxyCacheInvalidator,
{
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider> {
        let provider = self.inner.create_provider(input).await?;
        self.refresh_scheduling().await?;
        Ok(provider)
    }

    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider> {
        let provider = self.inner.update_provider(id, input).await?;
        self.refresh_scheduling().await?;
        Ok(provider)
    }

    async fn delete_provider(&self, id: &str) -> ProviderResult<()> {
        self.inner.delete_provider(id).await?;
        let cooldown_result = self.cache.clear_provider_cooldown(id).await;
        let scheduling_result = self.cache.refresh_scheduling().await;
        combine_cache_results(cooldown_result, scheduling_result).map_err(cache_error)
    }

    async fn find_provider(&self, id_or_name: &str) -> ProviderResult<Option<Provider>> {
        self.inner.find_provider(id_or_name).await
    }

    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse> {
        self.inner.list_providers(request).await
    }

    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint> {
        let endpoint = self.inner.create_endpoint(provider_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(endpoint)
    }

    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint> {
        let endpoint = self.inner.update_endpoint(provider_id, endpoint_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(endpoint)
    }

    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()> {
        self.inner.delete_endpoint(provider_id, endpoint_id).await?;
        self.refresh_scheduling().await
    }

    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>> {
        self.inner.list_endpoints(provider_id).await
    }

    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate, encrypted_api_key: String) -> ProviderResult<ProviderApiKey> {
        let key = self.inner.create_api_key(provider_id, input, encrypted_api_key).await?;
        self.refresh_scheduling().await?;
        Ok(key)
    }

    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>> {
        self.inner.list_api_keys(provider_id).await
    }

    async fn list_api_key_secrets(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKeySecret>> {
        self.inner.list_api_key_secrets(provider_id).await
    }

    async fn update_api_key(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderApiKeyUpdate,
        encrypted_api_key: Option<String>,
    ) -> ProviderResult<ProviderApiKey> {
        let key = self.inner.update_api_key(provider_id, key_id, input, encrypted_api_key).await?;
        self.refresh_scheduling().await?;
        Ok(key)
    }

    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()> {
        self.inner.delete_api_key(provider_id, key_id).await?;
        self.refresh_scheduling().await
    }

    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding> {
        let binding = self.inner.create_model_binding(provider_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(binding)
    }

    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>> {
        self.inner.list_model_bindings(provider_id).await
    }

    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding> {
        let binding = self.inner.update_model_binding(provider_id, model_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(binding)
    }

    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()> {
        self.inner.delete_model_binding(provider_id, model_id).await?;
        self.refresh_scheduling().await
    }

    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse> {
        self.inner.list_model_costs(provider_id).await
    }

    async fn upsert_model_costs(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderModelCostBatchUpsert,
    ) -> ProviderResult<ProviderModelCostListResponse> {
        let response = self.inner.upsert_model_costs(provider_id, key_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(response)
    }

    async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()> {
        self.inner.delete_model_cost(provider_id, key_id, provider_model_id).await?;
        self.refresh_scheduling().await
    }

    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse> {
        self.inner.list_request_records(request).await
    }

    async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> ProviderResult<UsageRecordListResponse> {
        self.inner.list_usage_records(user_id, request).await
    }

    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse> {
        self.inner.list_active_request_records(request).await
    }

    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail> {
        self.inner.get_request_record(request_id).await
    }

    async fn list_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> ProviderResult<ProviderCooldownListResponse> {
        self.inner.list_provider_cooldowns(request).await
    }

    async fn release_provider_cooldown(&self, provider_id: &str) -> ProviderResult<ProviderCooldown> {
        let cooldown = self.inner.release_provider_cooldown(provider_id).await?;
        self.cache.clear_provider_cooldown(provider_id).await.map_err(cache_error)?;
        Ok(cooldown)
    }
}

impl<R, C> CachedProviderRepository<R, C>
where
    C: ProxyCacheInvalidator,
{
    async fn refresh_scheduling(&self) -> ProviderResult<()> {
        self.cache.refresh_scheduling().await.map_err(cache_error)
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> ProviderError {
    ProviderError::Infrastructure(error.to_string())
}
