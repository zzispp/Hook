use async_trait::async_trait;
use storage::{
    Database, StorageError,
    model::ModelStore,
    provider::{
        ProviderApiKeyRecordInput, ProviderApiKeyRecordPatch, ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderModelRecordInput,
        ProviderModelRecordPatch, ProviderRecordInput, ProviderRecordPatch, ProviderStore,
    },
};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCreate,
    ProviderEndpoint, ProviderEndpointCreate, ProviderEndpointUpdate, ProviderListRequest, ProviderListResponse, ProviderModelBinding,
    ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderUpdate, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse,
};

use crate::application::{GlobalModelCatalog, ProviderApiKeySecret, ProviderError, ProviderRepository, ProviderResult};

const DEFAULT_PROVIDER_MAX_RETRIES: i32 = 2;
const DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS: f64 = 60.0;

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
                        name: record.name,
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

    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse> {
        self.store.list_request_records(request).await.map_err(storage_error)
    }

    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse> {
        self.store.list_active_request_records(request).await.map_err(storage_error)
    }

    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail> {
        self.store.get_request_record(request_id).await.map_err(storage_error)
    }
}

#[async_trait]
impl GlobalModelCatalog for StorageGlobalModelCatalog {
    async fn global_model_exists(&self, id: &str) -> ProviderResult<bool> {
        self.store.get_global_model(id).await.map(|model| model.is_some()).map_err(storage_error)
    }
}

fn provider_input(input: ProviderCreate) -> ProviderRecordInput {
    ProviderRecordInput {
        name: input.name,
        provider_type: input.provider_type,
        max_retries: Some(input.max_retries.unwrap_or(DEFAULT_PROVIDER_MAX_RETRIES)),
        request_timeout_seconds: Some(input.request_timeout_seconds.unwrap_or(DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS)),
        stream_first_byte_timeout_seconds: Some(
            input
                .stream_first_byte_timeout_seconds
                .unwrap_or(DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS),
        ),
        priority: input.priority.unwrap_or(100),
        keep_priority_on_conversion: input.keep_priority_on_conversion.unwrap_or(false),
        enable_format_conversion: input.enable_format_conversion.unwrap_or(false),
        is_active: input.is_active.unwrap_or(true),
    }
}

fn provider_patch(input: ProviderUpdate) -> ProviderRecordPatch {
    ProviderRecordPatch {
        name: input.name,
        provider_type: input.provider_type,
        max_retries: input.max_retries,
        request_timeout_seconds: input.request_timeout_seconds,
        stream_first_byte_timeout_seconds: input.stream_first_byte_timeout_seconds,
        priority: input.priority,
        keep_priority_on_conversion: input.keep_priority_on_conversion,
        enable_format_conversion: input.enable_format_conversion,
        is_active: input.is_active,
    }
}

fn endpoint_input(provider_id: &str, input: ProviderEndpointCreate) -> ProviderEndpointRecordInput {
    ProviderEndpointRecordInput {
        provider_id: provider_id.to_owned(),
        api_format: input.api_format,
        base_url: input.base_url,
        custom_path: input.custom_path,
        max_retries: input.max_retries,
        is_active: input.is_active.unwrap_or(true),
        format_acceptance_config: input.format_acceptance_config,
        header_rules: input.header_rules,
        body_rules: input.body_rules,
    }
}

fn endpoint_patch(input: ProviderEndpointUpdate) -> ProviderEndpointRecordPatch {
    ProviderEndpointRecordPatch {
        api_format: input.api_format,
        base_url: input.base_url,
        custom_path: input.custom_path,
        max_retries: input.max_retries,
        is_active: input.is_active,
        format_acceptance_config: input.format_acceptance_config,
        header_rules: input.header_rules,
        body_rules: input.body_rules,
    }
}

fn api_key_input(provider_id: &str, input: ProviderApiKeyCreate, encrypted_api_key: String) -> ProviderApiKeyRecordInput {
    ProviderApiKeyRecordInput {
        provider_id: provider_id.to_owned(),
        name: input.name,
        encrypted_api_key,
        note: input.note,
        internal_priority: input.internal_priority.unwrap_or(10),
        rpm_limit: input.rpm_limit,
        cache_ttl_minutes: input.cache_ttl_minutes.unwrap_or(5),
        max_probe_interval_minutes: input.max_probe_interval_minutes.unwrap_or(32),
        time_range_enabled: input.time_range_enabled.unwrap_or(false),
        time_range_start: input.time_range_start,
        time_range_end: input.time_range_end,
        is_active: input.is_active.unwrap_or(true),
    }
}

fn api_key_patch(input: ProviderApiKeyUpdate, encrypted_api_key: Option<String>) -> ProviderApiKeyRecordPatch {
    ProviderApiKeyRecordPatch {
        name: input.name,
        encrypted_api_key,
        note: input.note,
        internal_priority: input.internal_priority,
        rpm_limit: input.rpm_limit,
        cache_ttl_minutes: input.cache_ttl_minutes,
        max_probe_interval_minutes: input.max_probe_interval_minutes,
        time_range_enabled: input.time_range_enabled,
        time_range_start: input.time_range_start,
        time_range_end: input.time_range_end,
        is_active: input.is_active,
    }
}

fn model_binding_input(provider_id: &str, input: ProviderModelBindingCreate) -> ProviderModelRecordInput {
    ProviderModelRecordInput {
        provider_id: provider_id.to_owned(),
        global_model_id: input.global_model_id,
        provider_model_name: input.provider_model_name,
        provider_model_mapping: input.provider_model_mapping,
        is_active: true,
        price_per_request: None,
        tiered_pricing: None,
        config: input.config,
    }
}

fn model_binding_patch(input: ProviderModelBindingUpdate) -> ProviderModelRecordPatch {
    ProviderModelRecordPatch {
        provider_model_name: input.provider_model_name,
        is_active: input.is_active,
        provider_model_mapping: input.provider_model_mapping,
        config: input.config,
    }
}

fn storage_error(error: StorageError) -> ProviderError {
    match error {
        StorageError::NotFound => ProviderError::NotFound,
        StorageError::Conflict(message) => ProviderError::Conflict(message),
        StorageError::Database(message) => ProviderError::Infrastructure(message),
    }
}
