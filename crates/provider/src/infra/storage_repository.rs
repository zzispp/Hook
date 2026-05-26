use async_trait::async_trait;
use storage::{Database, StorageError, model::ModelStore, provider::ProviderStore};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCooldown,
    ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint, ProviderEndpointCreate, ProviderEndpointUpdate,
    ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderModelCostBatchUpsert,
    ProviderModelCostListResponse, ProviderUpdate, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse, UsageRecordListResponse,
};

use crate::application::{GlobalModelCatalog, ProviderApiKeySecret, ProviderError, ProviderRepository, ProviderResult};
use crate::infra::storage_mapping::{
    api_key_input, api_key_patch, endpoint_input, endpoint_patch, model_binding_input, model_binding_patch, model_cost_inputs, provider_input,
    provider_patch,
};

#[derive(Clone)]
pub struct StorageProviderRepository {
    store: ProviderStore,
}

#[derive(Clone)]
pub struct StorageGlobalModelCatalog {
    store: ModelStore,
}

impl StorageProviderRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: ProviderStore::new(database),
        }
    }
}

impl StorageGlobalModelCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: ModelStore::new(database),
        }
    }
}

#[async_trait]
impl ProviderRepository for StorageProviderRepository {
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider> {
        self.store.create_provider(provider_input(input)).await.map_err(storage_error)
    }

    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider> {
        self.store.update_provider(id, provider_patch(input)).await.map_err(storage_error)
    }

    async fn delete_provider(&self, id: &str) -> ProviderResult<()> {
        self.store.delete_provider(id).await.map_err(storage_error)
    }

    async fn find_provider(&self, id_or_name: &str) -> ProviderResult<Option<Provider>> {
        self.store.find_provider(id_or_name).await.map_err(storage_error)
    }

    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse> {
        self.store.list_providers(request).await.map_err(storage_error)
    }

    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint> {
        self.store.create_endpoint(endpoint_input(provider_id, input)).await.map_err(storage_error)
    }

    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint> {
        self.store
            .update_endpoint(provider_id, endpoint_id, endpoint_patch(input))
            .await
            .map_err(storage_error)
    }

    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()> {
        self.store.delete_endpoint(provider_id, endpoint_id).await.map_err(storage_error)
    }

    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>> {
        self.store.endpoints_for_provider(provider_id).await.map_err(storage_error)
    }

    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate, encrypted_api_key: String) -> ProviderResult<ProviderApiKey> {
        self.store
            .create_api_key(api_key_input(provider_id, input, encrypted_api_key))
            .await
            .map_err(storage_error)
    }

    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>> {
        self.store.api_keys_for_provider(provider_id).await.map_err(storage_error)
    }

    async fn list_api_key_secrets(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKeySecret>> {
        self.store
            .api_key_secrets_for_provider(provider_id)
            .await
            .map(|records| {
                records
                    .into_iter()
                    .map(|record| ProviderApiKeySecret {
                        id: record.id,
                        name: record.name,
                        api_formats: record.api_formats,
                        allowed_model_ids: record.allowed_model_ids,
                        encrypted_api_key: record.encrypted_api_key,
                        internal_priority: record.internal_priority,
                        is_active: record.is_active,
                    })
                    .collect()
            })
            .map_err(storage_error)
    }

    async fn update_api_key(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderApiKeyUpdate,
        encrypted_api_key: Option<String>,
    ) -> ProviderResult<ProviderApiKey> {
        self.store
            .update_api_key(provider_id, key_id, api_key_patch(input, encrypted_api_key))
            .await
            .map_err(storage_error)
    }

    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()> {
        self.store.delete_api_key(provider_id, key_id).await.map_err(storage_error)
    }

    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding> {
        self.store
            .create_model_binding(model_binding_input(provider_id, input))
            .await
            .map_err(storage_error)
    }

    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>> {
        self.store.model_bindings_for_provider(provider_id).await.map_err(storage_error)
    }

    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding> {
        self.store
            .update_model_binding(provider_id, model_id, model_binding_patch(input))
            .await
            .map_err(storage_error)
    }

    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()> {
        self.store.delete_model_binding(provider_id, model_id).await.map_err(storage_error)
    }

    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse> {
        self.store
            .list_model_costs(provider_id)
            .await
            .map(|costs| ProviderModelCostListResponse { costs })
            .map_err(storage_error)
    }

    async fn upsert_model_costs(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderModelCostBatchUpsert,
    ) -> ProviderResult<ProviderModelCostListResponse> {
        self.store
            .upsert_model_costs(model_cost_inputs(provider_id, key_id, input))
            .await
            .map(|costs| ProviderModelCostListResponse { costs })
            .map_err(storage_error)
    }

    async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()> {
        self.store
            .delete_model_cost(provider_id, key_id, provider_model_id)
            .await
            .map_err(storage_error)
    }

    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse> {
        self.store.list_request_records(request).await.map_err(storage_error)
    }

    async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> ProviderResult<UsageRecordListResponse> {
        self.store.list_usage_records(user_id, request).await.map_err(storage_error)
    }

    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse> {
        self.store.list_active_request_records(request).await.map_err(storage_error)
    }

    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail> {
        self.store.get_request_record(request_id).await.map_err(storage_error)
    }

    async fn list_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> ProviderResult<ProviderCooldownListResponse> {
        self.store.list_active_provider_cooldowns(request).await.map_err(storage_error)
    }

    async fn release_provider_cooldown(&self, provider_id: &str) -> ProviderResult<ProviderCooldown> {
        self.store.release_provider_cooldown(provider_id).await.map_err(storage_error)
    }
}

#[async_trait]
impl GlobalModelCatalog for StorageGlobalModelCatalog {
    async fn global_model_exists(&self, id: &str) -> ProviderResult<bool> {
        self.store.get_global_model(id).await.map(|model| model.is_some()).map_err(storage_error)
    }
}

fn storage_error(error: StorageError) -> ProviderError {
    match error {
        StorageError::NotFound => ProviderError::NotFound,
        StorageError::Conflict(message) => ProviderError::Conflict(message),
        StorageError::Database(message) => ProviderError::Infrastructure(message),
    }
}
