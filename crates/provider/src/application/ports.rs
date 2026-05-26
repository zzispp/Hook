use async_trait::async_trait;
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCooldown,
    ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint, ProviderEndpointCreate, ProviderEndpointUpdate,
    ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderModelCostBatchUpsert,
    ProviderModelCostListResponse, ProviderModelTestRequest, ProviderModelTestResponse, ProviderUpdate, ProviderUpstreamModelsResponse, RequestRecordDetail,
    RequestRecordListRequest, RequestRecordListResponse, UsageRecordListResponse,
};

use super::ProviderResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderApiKeySecret {
    pub id: String,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub encrypted_api_key: String,
    pub internal_priority: i32,
    pub is_active: bool,
}

#[async_trait]
pub trait ProviderRepository: Send + Sync + 'static {
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider>;
    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider>;
    async fn delete_provider(&self, id: &str) -> ProviderResult<()>;
    async fn find_provider(&self, id_or_name: &str) -> ProviderResult<Option<Provider>>;
    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse>;
    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint>;
    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint>;
    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()>;
    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>>;
    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate, encrypted_api_key: String) -> ProviderResult<ProviderApiKey>;
    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>>;
    async fn list_api_key_secrets(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKeySecret>>;
    async fn update_api_key(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderApiKeyUpdate,
        encrypted_api_key: Option<String>,
    ) -> ProviderResult<ProviderApiKey>;
    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()>;
    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding>;
    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding>;
    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()>;
    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse>;
    async fn upsert_model_costs(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderModelCostBatchUpsert,
    ) -> ProviderResult<ProviderModelCostListResponse>;
    async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()>;
    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse>;
    async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> ProviderResult<UsageRecordListResponse>;
    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse>;
    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail>;
    async fn list_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> ProviderResult<ProviderCooldownListResponse>;
    async fn release_provider_cooldown(&self, provider_id: &str) -> ProviderResult<ProviderCooldown>;
}

#[async_trait]
pub trait GlobalModelCatalog: Send + Sync + 'static {
    async fn global_model_exists(&self, id: &str) -> ProviderResult<bool>;
}

pub trait SecretCipher: Send + Sync + 'static {
    fn encrypt_provider_key(&self, plaintext: &str) -> ProviderResult<String>;
    fn decrypt_provider_key(&self, ciphertext: &str) -> ProviderResult<String>;
}

#[async_trait]
pub trait UpstreamModelFetcher: Send + Sync + 'static {
    async fn fetch_upstream_models(&self, endpoint: &ProviderEndpoint, api_key: &str) -> ProviderResult<ProviderUpstreamModelsResponse>;
}

#[async_trait]
pub trait ProviderModelTester: Send + Sync + 'static {
    async fn test_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelTestRequest) -> ProviderResult<ProviderModelTestResponse>;
}

#[async_trait]
pub trait ProviderUseCase: Send + Sync + 'static {
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider>;
    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider>;
    async fn delete_provider(&self, id: &str) -> ProviderResult<()>;
    async fn get_provider(&self, id: &str) -> ProviderResult<Provider>;
    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse>;
    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint>;
    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint>;
    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()>;
    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>>;
    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate) -> ProviderResult<ProviderApiKey>;
    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>>;
    async fn fetch_upstream_models(&self, provider_id: &str) -> ProviderResult<ProviderUpstreamModelsResponse>;
    async fn update_api_key(&self, provider_id: &str, key_id: &str, input: ProviderApiKeyUpdate) -> ProviderResult<ProviderApiKey>;
    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()>;
    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding>;
    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding>;
    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()>;
    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse>;
    async fn upsert_model_costs(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderModelCostBatchUpsert,
    ) -> ProviderResult<ProviderModelCostListResponse>;
    async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()>;
    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse>;
    async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> ProviderResult<UsageRecordListResponse>;
    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse>;
    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail>;
    async fn list_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> ProviderResult<ProviderCooldownListResponse>;
    async fn release_provider_cooldown(&self, provider_id: &str) -> ProviderResult<ProviderCooldown>;
}
