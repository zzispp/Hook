use async_trait::async_trait;
use storage::{Database, StorageError, model::ModelStore, provider::ProviderStore};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyPriorityBatchUpdate,
    ProviderApiKeyUpdate, ProviderCooldown, ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint,
    ProviderEndpointCreate, ProviderEndpointUpdate, ProviderKeyGroup, ProviderKeyGroupCreate, ProviderKeyGroupListRequest, ProviderKeyGroupListResponse,
    ProviderKeyGroupUpdate, ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingBatchUpdate, ProviderModelBindingCreate,
    ProviderModelBindingUpdate, ProviderModelCostBatchUpsert, ProviderModelCostListResponse, ProviderUpdate, RequestRecordDetail, RequestRecordListRequest,
    RequestRecordListResponse, UsageRecordListResponse,
};

use crate::application::{
    GlobalModelCatalog, ProviderApiKeySecret, ProviderError, ProviderQuickImportAppend, ProviderQuickImportAppended, ProviderQuickImportBind,
    ProviderQuickImportBound, ProviderQuickImportCreate, ProviderQuickImportCreated, ProviderQuickImportKeyReplaced, ProviderQuickImportKeyReplacement,
    ProviderQuickImportSyncEventCreate, ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, ProviderQuickImportSyncKeyPatch,
    ProviderQuickImportSyncSource, ProviderQuickImportSyncSourcePatch, ProviderRepository, ProviderResult,
};
use crate::infra::storage_mapping::{
    api_key_input, api_key_patch, endpoint_input, endpoint_patch, model_binding_batch_input, model_binding_input, model_binding_patch, model_cost_inputs,
    provider_input, provider_key_group_input, provider_key_group_patch, provider_patch, quick_import_append_input, quick_import_bind_input, quick_import_input,
    quick_import_key_replacement_input,
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

    async fn provider_key_exists(&self, id: &str) -> ProviderResult<bool> {
        self.store.find_api_key(id).await.map(|key| key.is_some()).map_err(storage_error)
    }

    async fn create_provider_key_group(&self, input: ProviderKeyGroupCreate) -> ProviderResult<ProviderKeyGroup> {
        self.store
            .create_provider_key_group(provider_key_group_input(input))
            .await
            .map_err(storage_error)
    }

    async fn update_provider_key_group(&self, id: &str, input: ProviderKeyGroupUpdate) -> ProviderResult<ProviderKeyGroup> {
        self.store
            .update_provider_key_group(id, provider_key_group_patch(input))
            .await
            .map_err(storage_error)
    }

    async fn delete_provider_key_group(&self, id: &str) -> ProviderResult<()> {
        self.store.delete_provider_key_group(id).await.map_err(storage_error)
    }

    async fn find_provider_key_group(&self, id_or_name: &str) -> ProviderResult<Option<ProviderKeyGroup>> {
        self.store.find_provider_key_group(id_or_name).await.map_err(storage_error)
    }

    async fn list_provider_key_groups(&self, request: ProviderKeyGroupListRequest) -> ProviderResult<ProviderKeyGroupListResponse> {
        self.store.list_provider_key_groups(request).await.map_err(storage_error)
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
                        capabilities: record.capabilities,
                        encrypted_api_key: record.encrypted_api_key,
                        internal_priority: record.internal_priority,
                        global_priority_by_format: record.global_priority_by_format,
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

    async fn batch_update_api_key_priorities(&self, input: ProviderApiKeyPriorityBatchUpdate) -> ProviderResult<Vec<ProviderApiKey>> {
        self.store
            .batch_update_api_key_priorities(
                input
                    .updates
                    .into_iter()
                    .map(|update| storage::provider::ProviderApiKeyPriorityRecordPatch {
                        provider_id: update.provider_id,
                        key_id: update.key_id,
                        global_priority_by_format: update.global_priority_by_format,
                    })
                    .collect(),
            )
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

    async fn batch_update_model_bindings(&self, provider_id: &str, input: ProviderModelBindingBatchUpdate) -> ProviderResult<Vec<ProviderModelBinding>> {
        self.store
            .batch_update_model_bindings(model_binding_batch_input(provider_id, input))
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

    async fn upsert_model_costs(&self, provider_id: &str, key_id: &str, input: ProviderModelCostBatchUpsert) -> ProviderResult<ProviderModelCostListResponse> {
        self.store
            .upsert_model_costs(model_cost_inputs(provider_id, key_id, input))
            .await
            .map(|costs| ProviderModelCostListResponse { costs })
            .map_err(storage_error)
    }

    async fn create_quick_import(&self, input: ProviderQuickImportCreate) -> ProviderResult<ProviderQuickImportCreated> {
        self.store
            .create_quick_import(quick_import_input(input))
            .await
            .map(|output| ProviderQuickImportCreated {
                provider: output.provider,
                endpoints: output.endpoints,
                api_keys: output.api_keys,
                model_bindings: output.model_bindings,
                model_costs: output.model_costs,
            })
            .map_err(storage_error)
    }

    async fn append_quick_import(&self, input: ProviderQuickImportAppend) -> ProviderResult<ProviderQuickImportAppended> {
        self.store
            .append_quick_import(quick_import_append_input(input))
            .await
            .map(|output| ProviderQuickImportAppended {
                endpoints: output.endpoints,
                api_keys: output.api_keys,
                model_bindings: output.model_bindings,
                model_costs: output.model_costs,
            })
            .map_err(storage_error)
    }

    async fn bind_quick_import(&self, input: ProviderQuickImportBind) -> ProviderResult<ProviderQuickImportBound> {
        self.store
            .bind_quick_import(quick_import_bind_input(input))
            .await
            .map(|output| ProviderQuickImportBound {
                provider: output.provider,
                endpoints: output.endpoints,
                api_keys: output.api_keys,
                model_bindings: output.model_bindings,
                model_costs: output.model_costs,
                created_key_count: output.created_key_count,
                reused_key_count: output.reused_key_count,
                deleted_key_count: output.deleted_key_count,
            })
            .map_err(storage_error)
    }

    async fn replace_quick_import_key(&self, input: ProviderQuickImportKeyReplacement) -> ProviderResult<ProviderQuickImportKeyReplaced> {
        self.store
            .replace_quick_import_key(quick_import_key_replacement_input(input))
            .await
            .map(|output| ProviderQuickImportKeyReplaced {
                api_key: output.api_key,
                model_bindings: output.model_bindings,
                model_costs: output.model_costs,
            })
            .map_err(storage_error)
    }

    async fn quick_import_sync_source(&self, provider_id: &str) -> ProviderResult<Option<ProviderQuickImportSyncSource>> {
        self.store
            .quick_import_source_for_provider(provider_id)
            .await
            .map(|source| source.map(sync_source))
            .map_err(storage_error)
    }

    async fn list_quick_import_sync_sources(&self, limit: u64) -> ProviderResult<Vec<ProviderQuickImportSyncSource>> {
        self.store
            .list_quick_import_sync_sources(limit)
            .await
            .map(|items| items.into_iter().map(sync_source).collect())
            .map_err(storage_error)
    }

    async fn list_quick_import_sync_keys(&self, source_id: &str) -> ProviderResult<Vec<ProviderQuickImportSyncKey>> {
        self.store
            .quick_import_sync_keys(source_id)
            .await
            .map(|items| items.into_iter().map(sync_key).collect())
            .map_err(storage_error)
    }

    async fn quick_import_sync_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<Option<ProviderQuickImportSyncKey>> {
        self.store
            .quick_import_sync_key(provider_id, key_id)
            .await
            .map(|item| item.map(sync_key))
            .map_err(storage_error)
    }

    async fn update_quick_import_sync_source(
        &self,
        provider_id: &str,
        input: ProviderQuickImportSyncSourcePatch,
    ) -> ProviderResult<ProviderQuickImportSyncSource> {
        self.store
            .update_quick_import_source(provider_id, sync_source_patch(input))
            .await
            .map(sync_source)
            .map_err(storage_error)
    }

    async fn update_quick_import_sync_source_run(
        &self,
        source_id: &str,
        status: Option<types::provider::ProviderQuickImportSyncStatus>,
        error: Option<String>,
        failed: bool,
    ) -> ProviderResult<()> {
        self.store
            .update_quick_import_source_run(source_id, status, error, failed)
            .await
            .map_err(storage_error)
    }

    async fn update_quick_import_sync_keys(&self, provider_id: &str, input: Vec<ProviderQuickImportSyncKeyPatch>) -> ProviderResult<()> {
        self.store
            .update_quick_import_sync_keys(provider_id, input.into_iter().map(sync_key_patch).collect())
            .await
            .map_err(storage_error)
    }

    async fn create_quick_import_sync_events(&self, input: Vec<ProviderQuickImportSyncEventCreate>) -> ProviderResult<()> {
        self.store
            .create_quick_import_sync_events(input.into_iter().map(sync_event_input).collect())
            .await
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

fn sync_source(record: storage::provider::ProviderQuickImportSourceRecord) -> ProviderQuickImportSyncSource {
    ProviderQuickImportSyncSource {
        id: record.id,
        provider_id: record.provider_id,
        provider_name: record.provider_name,
        source_kind: types::provider::ProviderQuickImportSourceKind::try_from(record.source_kind.as_str()).expect("quick import source kind must be valid"),
        base_url: record.base_url,
        encrypted_system_access_token: record.encrypted_system_access_token,
        email: record.email,
        encrypted_password: record.encrypted_password,
        encrypted_auth_token: record.encrypted_auth_token,
        encrypted_refresh_token: record.encrypted_refresh_token,
        token_expires_at: record.token_expires_at,
        user_id: record.user_id,
        recharge_multiplier: record.recharge_multiplier,
        sync_config: record.sync_config,
        last_status: record.last_status,
        last_error: record.last_error,
        last_synced_at: record.last_synced_at,
        consecutive_failures: record.consecutive_failures,
    }
}

fn sync_source_patch(input: ProviderQuickImportSyncSourcePatch) -> storage::provider::ProviderQuickImportSourceRecordPatch {
    storage::provider::ProviderQuickImportSourceRecordPatch {
        base_url: input.base_url,
        encrypted_system_access_token: input.encrypted_system_access_token,
        email: input.email,
        encrypted_password: input.encrypted_password,
        encrypted_auth_token: input.encrypted_auth_token,
        encrypted_refresh_token: input.encrypted_refresh_token,
        token_expires_at: input.token_expires_at,
        user_id: input.user_id,
        recharge_multiplier: input.recharge_multiplier,
        sync_config: input.sync_config,
    }
}

fn sync_key(record: storage::provider::ProviderQuickImportSyncKeyRecord) -> ProviderQuickImportSyncKey {
    ProviderQuickImportSyncKey {
        provider_id: record.provider_id,
        source_id: record.source_id,
        key_id: record.key_id,
        local_key_name: record.local_key_name,
        upstream_token_id: record.upstream_token_id,
        upstream_token_name: record.upstream_token_name,
        upstream_group: record.upstream_group,
        upstream_group_ratio: record.upstream_group_ratio,
        effective_cost_multiplier: record.effective_cost_multiplier,
        statuses: record.statuses,
        model_mappings: record.model_mappings.into_iter().map(sync_key_model).collect(),
    }
}

fn sync_key_model(record: storage::provider::ProviderQuickImportSyncKeyModelRecord) -> ProviderQuickImportSyncKeyModel {
    ProviderQuickImportSyncKeyModel {
        upstream_model_id: record.upstream_model_id,
        global_model_id: record.global_model_id,
    }
}

fn sync_key_patch(input: ProviderQuickImportSyncKeyPatch) -> storage::provider::ProviderQuickImportSyncKeyRecordPatch {
    storage::provider::ProviderQuickImportSyncKeyRecordPatch {
        key_id: input.key_id,
        statuses: input.statuses,
        upstream_group: input.upstream_group,
        upstream_group_ratio: input.upstream_group_ratio,
        effective_cost_multiplier: input.effective_cost_multiplier,
        last_error: input.last_error,
    }
}

fn sync_event_input(input: ProviderQuickImportSyncEventCreate) -> storage::provider::ProviderQuickImportSyncEventRecordInput {
    storage::provider::ProviderQuickImportSyncEventRecordInput {
        provider_id: input.provider_id,
        source_id: input.source_id,
        key_id: input.key_id,
        status: input.status,
        title: input.title,
        detail: input.detail,
    }
}

#[async_trait]
impl GlobalModelCatalog for StorageGlobalModelCatalog {
    async fn global_model_exists(&self, id: &str) -> ProviderResult<bool> {
        self.store.get_global_model(id).await.map(|model| model.is_some()).map_err(storage_error)
    }

    async fn list_global_models(&self) -> ProviderResult<Vec<types::model::GlobalModelResponse>> {
        self.store
            .list_global_models(types::model::GlobalModelListRequest {
                skip: 0,
                limit: u64::MAX,
                is_active: Some(true),
                search: None,
            })
            .await
            .map(|response| response.models)
            .map_err(storage_error)
    }
}

fn storage_error(error: StorageError) -> ProviderError {
    match error {
        StorageError::NotFound => ProviderError::NotFound,
        StorageError::Conflict(message) => ProviderError::Conflict(message),
        StorageError::Database(message) => ProviderError::Infrastructure(message),
    }
}
