use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use types::model::GlobalModelResponse;
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyPriorityBatchUpdate,
    ProviderApiKeyUpdate, ProviderCooldown, ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint,
    ProviderEndpointCreate, ProviderEndpointUpdate, ProviderKeyGroup, ProviderKeyGroupCreate, ProviderKeyGroupListRequest, ProviderKeyGroupListResponse,
    ProviderKeyGroupUpdate, ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingBatchUpdate, ProviderModelBindingCreate,
    ProviderModelBindingUpdate, ProviderModelCost, ProviderModelCostBatchUpsert, ProviderModelCostListResponse, ProviderModelCostUpsert,
    ProviderModelTestRequest, ProviderModelTestResponse, ProviderQuickImportAppendCommitRequest, ProviderQuickImportAppendPreviewRequest,
    ProviderQuickImportBindCommitRequest, ProviderQuickImportBindCommitResponse, ProviderQuickImportBindPreviewRequest, ProviderQuickImportBindPreviewResponse,
    ProviderQuickImportCommitRequest, ProviderQuickImportCommitResponse, ProviderQuickImportModelAssociationsResponse,
    ProviderQuickImportModelAssociationsUpdate, ProviderQuickImportPreviewRequest, ProviderQuickImportPreviewResponse, ProviderQuickImportRelinkRequest,
    ProviderQuickImportResolutionResponse, ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind, ProviderQuickImportSyncConfig,
    ProviderQuickImportSyncSettingsResponse, ProviderQuickImportSyncSettingsUpdate, ProviderQuickImportSyncStatus, ProviderUpdate,
    ProviderUpstreamModelsResponse, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse, UsageRecordListResponse,
};

use super::ProviderResult;

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderApiKeySecret {
    pub id: String,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub capabilities: Option<serde_json::Value>,
    pub encrypted_api_key: String,
    pub internal_priority: i32,
    pub global_priority_by_format: BTreeMap<String, i32>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpstreamImportData {
    pub source_kind: ProviderQuickImportSourceKind,
    pub tokens: Vec<UpstreamImportToken>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpstreamImportToken {
    pub id: String,
    pub name: String,
    pub masked_key: String,
    pub status: i32,
    pub group: Option<String>,
    pub group_ratio: Decimal,
    pub api_key: Option<String>,
    pub models: Vec<UpstreamImportModel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamImportModel {
    pub id: String,
    pub supported_endpoint_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpstreamSyncSnapshot {
    pub source_kind: ProviderQuickImportSourceKind,
    pub groups: BTreeMap<String, UpstreamGroupRatio>,
    pub tokens: Vec<UpstreamSyncToken>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpstreamGroupRatio {
    Fixed(Decimal),
    UpstreamValue(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamSyncToken {
    pub id: String,
    pub name: String,
    pub masked_key: String,
    pub status: i32,
    pub group: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportCreate {
    pub provider: ProviderCreate,
    pub sync_source: Option<ProviderQuickImportSyncSourceCreate>,
    pub endpoints: Vec<ProviderEndpointCreate>,
    pub model_bindings: Vec<ProviderModelBindingCreate>,
    pub api_keys: Vec<ProviderQuickImportApiKeyCreate>,
    pub model_costs: Vec<ProviderQuickImportModelCostCreate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSyncSourceCreate {
    pub source_kind: ProviderQuickImportSourceKind,
    pub base_url: String,
    pub encrypted_system_access_token: String,
    pub user_id: String,
    pub recharge_multiplier: Decimal,
    pub sync_config: ProviderQuickImportSyncConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportApiKeyCreate {
    pub upstream_token_id: String,
    pub upstream_token_name: String,
    pub upstream_masked_key: String,
    pub upstream_group: Option<String>,
    pub upstream_group_ratio: Decimal,
    pub effective_cost_multiplier: Decimal,
    pub model_mappings: Vec<ProviderQuickImportKeyModelCreate>,
    pub input: ProviderApiKeyCreate,
    pub encrypted_api_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportKeyModelCreate {
    pub upstream_model_id: String,
    pub global_model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportModelCostCreate {
    pub upstream_token_id: String,
    pub global_model_id: String,
    pub cost: ProviderModelCostUpsert,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportCreated {
    pub provider: Provider,
    pub endpoints: Vec<ProviderEndpoint>,
    pub api_keys: Vec<ProviderApiKey>,
    pub model_bindings: Vec<ProviderModelBinding>,
    pub model_costs: Vec<ProviderModelCost>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportAppend {
    pub provider_id: String,
    pub source_id: String,
    pub endpoints: Vec<ProviderEndpointCreate>,
    pub model_bindings: Vec<ProviderModelBindingCreate>,
    pub api_keys: Vec<ProviderQuickImportApiKeyCreate>,
    pub model_costs: Vec<ProviderQuickImportModelCostCreate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportAppended {
    pub endpoints: Vec<ProviderEndpoint>,
    pub api_keys: Vec<ProviderApiKey>,
    pub model_bindings: Vec<ProviderModelBinding>,
    pub model_costs: Vec<ProviderModelCost>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportBoundApiKey {
    pub local_key_id: Option<String>,
    pub create: ProviderQuickImportApiKeyCreate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportBind {
    pub provider_id: String,
    pub sync_source: ProviderQuickImportSyncSourceCreate,
    pub endpoints: Vec<ProviderEndpointCreate>,
    pub model_bindings: Vec<ProviderModelBindingCreate>,
    pub api_keys: Vec<ProviderQuickImportBoundApiKey>,
    pub model_costs: Vec<ProviderQuickImportModelCostCreate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportBound {
    pub provider: Provider,
    pub endpoints: Vec<ProviderEndpoint>,
    pub api_keys: Vec<ProviderApiKey>,
    pub model_bindings: Vec<ProviderModelBinding>,
    pub model_costs: Vec<ProviderModelCost>,
    pub created_key_count: usize,
    pub reused_key_count: usize,
    pub deleted_key_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportKeyReplacement {
    pub provider_id: String,
    pub source_id: String,
    pub key_id: String,
    pub upstream_token_id: String,
    pub upstream_token_name: String,
    pub upstream_masked_key: String,
    pub upstream_group: Option<String>,
    pub upstream_group_ratio: Decimal,
    pub effective_cost_multiplier: Decimal,
    pub model_mappings: Vec<ProviderQuickImportKeyModelCreate>,
    pub input: ProviderApiKeyUpdate,
    pub encrypted_api_key: Option<String>,
    pub model_bindings: Vec<ProviderModelBindingCreate>,
    pub model_costs: Vec<ProviderQuickImportModelCostCreate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportKeyReplaced {
    pub api_key: ProviderApiKey,
    pub model_bindings: Vec<ProviderModelBinding>,
    pub model_costs: Vec<ProviderModelCost>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSyncSource {
    pub id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub source_kind: ProviderQuickImportSourceKind,
    pub base_url: String,
    pub encrypted_system_access_token: String,
    pub user_id: String,
    pub recharge_multiplier: Decimal,
    pub sync_config: ProviderQuickImportSyncConfig,
    pub last_status: Option<ProviderQuickImportSyncStatus>,
    pub last_error: Option<String>,
    pub last_synced_at: Option<time::OffsetDateTime>,
    pub consecutive_failures: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderQuickImportSyncSourcePatch {
    pub base_url: Option<String>,
    pub encrypted_system_access_token: Option<String>,
    pub user_id: Option<String>,
    pub recharge_multiplier: Option<Decimal>,
    pub sync_config: Option<ProviderQuickImportSyncConfig>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSyncKey {
    pub provider_id: String,
    pub source_id: String,
    pub key_id: String,
    pub local_key_name: String,
    pub upstream_token_id: String,
    pub upstream_token_name: String,
    pub upstream_group: Option<String>,
    pub upstream_group_ratio: Decimal,
    pub effective_cost_multiplier: Decimal,
    pub statuses: Vec<ProviderQuickImportSyncStatus>,
    pub model_mappings: Vec<ProviderQuickImportSyncKeyModel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderQuickImportSyncKeyModel {
    pub upstream_model_id: String,
    pub global_model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSyncKeyPatch {
    pub key_id: String,
    pub statuses: Vec<ProviderQuickImportSyncStatus>,
    pub upstream_group: Option<Option<String>>,
    pub upstream_group_ratio: Option<Decimal>,
    pub effective_cost_multiplier: Option<Decimal>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderQuickImportSyncEventCreate {
    pub provider_id: String,
    pub source_id: String,
    pub key_id: Option<String>,
    pub status: ProviderQuickImportSyncStatus,
    pub title: String,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderQuickImportSyncRunOptions {
    pub limit: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderQuickImportSyncRunReport {
    pub scanned_count: u64,
    pub synced_count: u64,
    pub failed_count: u64,
    pub disabled_key_count: u64,
    pub updated_cost_count: u64,
}

#[async_trait]
pub trait ProviderRepository: Send + Sync + 'static {
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider>;
    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider>;
    async fn delete_provider(&self, id: &str) -> ProviderResult<()>;
    async fn find_provider(&self, id_or_name: &str) -> ProviderResult<Option<Provider>>;
    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse>;
    async fn provider_key_exists(&self, id: &str) -> ProviderResult<bool>;
    async fn create_provider_key_group(&self, input: ProviderKeyGroupCreate) -> ProviderResult<ProviderKeyGroup>;
    async fn update_provider_key_group(&self, id: &str, input: ProviderKeyGroupUpdate) -> ProviderResult<ProviderKeyGroup>;
    async fn delete_provider_key_group(&self, id: &str) -> ProviderResult<()>;
    async fn find_provider_key_group(&self, id_or_name: &str) -> ProviderResult<Option<ProviderKeyGroup>>;
    async fn list_provider_key_groups(&self, request: ProviderKeyGroupListRequest) -> ProviderResult<ProviderKeyGroupListResponse>;
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
    async fn batch_update_api_key_priorities(&self, input: ProviderApiKeyPriorityBatchUpdate) -> ProviderResult<Vec<ProviderApiKey>>;
    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()>;
    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding>;
    async fn batch_update_model_bindings(&self, provider_id: &str, input: ProviderModelBindingBatchUpdate) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding>;
    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()>;
    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse>;
    async fn upsert_model_costs(&self, provider_id: &str, key_id: &str, input: ProviderModelCostBatchUpsert) -> ProviderResult<ProviderModelCostListResponse>;
    async fn create_quick_import(&self, input: ProviderQuickImportCreate) -> ProviderResult<ProviderQuickImportCreated>;
    async fn append_quick_import(&self, input: ProviderQuickImportAppend) -> ProviderResult<ProviderQuickImportAppended>;
    async fn bind_quick_import(&self, input: ProviderQuickImportBind) -> ProviderResult<ProviderQuickImportBound>;
    async fn replace_quick_import_key(&self, input: ProviderQuickImportKeyReplacement) -> ProviderResult<ProviderQuickImportKeyReplaced>;
    async fn quick_import_sync_source(&self, provider_id: &str) -> ProviderResult<Option<ProviderQuickImportSyncSource>>;
    async fn list_quick_import_sync_sources(&self, limit: u64) -> ProviderResult<Vec<ProviderQuickImportSyncSource>>;
    async fn list_quick_import_sync_keys(&self, source_id: &str) -> ProviderResult<Vec<ProviderQuickImportSyncKey>>;
    async fn quick_import_sync_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<Option<ProviderQuickImportSyncKey>>;
    async fn update_quick_import_sync_source(
        &self,
        provider_id: &str,
        input: ProviderQuickImportSyncSourcePatch,
    ) -> ProviderResult<ProviderQuickImportSyncSource>;
    async fn update_quick_import_sync_source_run(
        &self,
        source_id: &str,
        status: Option<ProviderQuickImportSyncStatus>,
        error: Option<String>,
        failed: bool,
    ) -> ProviderResult<()>;
    async fn update_quick_import_sync_keys(&self, provider_id: &str, input: Vec<ProviderQuickImportSyncKeyPatch>) -> ProviderResult<()>;
    async fn create_quick_import_sync_events(&self, input: Vec<ProviderQuickImportSyncEventCreate>) -> ProviderResult<()>;
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
    async fn list_global_models(&self) -> ProviderResult<Vec<GlobalModelResponse>>;
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
pub trait UpstreamProviderImportSource: Send + Sync + 'static {
    async fn fetch_import_data(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData>;
    async fn fetch_sync_snapshot(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot>;
    async fn fetch_sync_token_models(&self, source: &ProviderQuickImportSourceConfig, upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>>;
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
    async fn create_provider_key_group(&self, input: ProviderKeyGroupCreate) -> ProviderResult<ProviderKeyGroup>;
    async fn update_provider_key_group(&self, id: &str, input: ProviderKeyGroupUpdate) -> ProviderResult<ProviderKeyGroup>;
    async fn delete_provider_key_group(&self, id: &str) -> ProviderResult<()>;
    async fn get_provider_key_group(&self, id: &str) -> ProviderResult<ProviderKeyGroup>;
    async fn list_provider_key_groups(&self, request: ProviderKeyGroupListRequest) -> ProviderResult<ProviderKeyGroupListResponse>;
    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint>;
    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint>;
    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()>;
    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>>;
    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate) -> ProviderResult<ProviderApiKey>;
    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>>;
    async fn fetch_upstream_models(&self, provider_id: &str) -> ProviderResult<ProviderUpstreamModelsResponse>;
    async fn update_api_key(&self, provider_id: &str, key_id: &str, input: ProviderApiKeyUpdate) -> ProviderResult<ProviderApiKey>;
    async fn batch_update_api_key_priorities(&self, input: ProviderApiKeyPriorityBatchUpdate) -> ProviderResult<Vec<ProviderApiKey>>;
    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()>;
    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding>;
    async fn batch_update_model_bindings(&self, provider_id: &str, input: ProviderModelBindingBatchUpdate) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>>;
    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding>;
    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()>;
    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse>;
    async fn upsert_model_costs(&self, provider_id: &str, key_id: &str, input: ProviderModelCostBatchUpsert) -> ProviderResult<ProviderModelCostListResponse>;
    async fn preview_quick_import(&self, input: ProviderQuickImportPreviewRequest) -> ProviderResult<ProviderQuickImportPreviewResponse>;
    async fn commit_quick_import(&self, input: ProviderQuickImportCommitRequest) -> ProviderResult<ProviderQuickImportCommitResponse>;
    async fn preview_quick_import_append(
        &self,
        provider_id: &str,
        input: ProviderQuickImportAppendPreviewRequest,
    ) -> ProviderResult<ProviderQuickImportPreviewResponse>;
    async fn commit_quick_import_append(
        &self,
        provider_id: &str,
        input: ProviderQuickImportAppendCommitRequest,
    ) -> ProviderResult<ProviderQuickImportCommitResponse>;
    async fn preview_quick_import_bind(
        &self,
        provider_id: &str,
        input: ProviderQuickImportBindPreviewRequest,
    ) -> ProviderResult<ProviderQuickImportBindPreviewResponse>;
    async fn commit_quick_import_bind(
        &self,
        provider_id: &str,
        input: ProviderQuickImportBindCommitRequest,
    ) -> ProviderResult<ProviderQuickImportBindCommitResponse>;
    async fn quick_import_resolution(&self, provider_id: &str, key_id: &str) -> ProviderResult<ProviderQuickImportResolutionResponse>;
    async fn accept_quick_import_current(&self, provider_id: &str, key_id: &str) -> ProviderResult<ProviderApiKey>;
    async fn relink_quick_import_key(&self, provider_id: &str, key_id: &str, input: ProviderQuickImportRelinkRequest) -> ProviderResult<ProviderApiKey>;
    async fn quick_import_model_associations(&self, provider_id: &str, key_id: &str) -> ProviderResult<ProviderQuickImportModelAssociationsResponse>;
    async fn update_quick_import_model_associations(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderQuickImportModelAssociationsUpdate,
    ) -> ProviderResult<ProviderQuickImportModelAssociationsResponse>;
    async fn quick_import_sync_settings(&self, provider_id: &str) -> ProviderResult<ProviderQuickImportSyncSettingsResponse>;
    async fn update_quick_import_sync_settings(
        &self,
        provider_id: &str,
        input: ProviderQuickImportSyncSettingsUpdate,
    ) -> ProviderResult<ProviderQuickImportSyncSettingsResponse>;
    async fn run_quick_import_sync(&self, options: ProviderQuickImportSyncRunOptions) -> ProviderResult<ProviderQuickImportSyncRunReport>;
    async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()>;
    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse>;
    async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> ProviderResult<UsageRecordListResponse>;
    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse>;
    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail>;
    async fn list_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> ProviderResult<ProviderCooldownListResponse>;
    async fn release_provider_cooldown(&self, provider_id: &str) -> ProviderResult<ProviderCooldown>;
}
