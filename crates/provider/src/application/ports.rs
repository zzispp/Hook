use async_trait::async_trait;
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderCreate, ProviderEndpoint,
    ProviderEndpointCreate, ProviderEndpointUpdate, ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingCreate,
    ProviderUpdate, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse,
};

use super::ProviderResult;

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
    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding>;
    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse>;
    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse>;
    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail>;
}

#[async_trait]
pub trait GlobalModelCatalog: Send + Sync + 'static {
    async fn global_model_exists(&self, id: &str) -> ProviderResult<bool>;
}

pub trait SecretCipher: Send + Sync + 'static {
    fn encrypt_provider_key(&self, plaintext: &str) -> ProviderResult<String>;
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
    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding>;
    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse>;
    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse>;
    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail>;
}
