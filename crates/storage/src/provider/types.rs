use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;
use types::model::PatchField;
use types::provider::{
    ProviderKeyGroupMemberInput, ProviderModelCostMode, ProviderModelCostSource, ProviderOrigin, ProviderQuickImportSyncConfig,
    ProviderQuickImportSyncEventPayload, ProviderQuickImportSyncStatus, RequestUpstreamCost,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderRecordInput {
    pub name: String,
    pub provider_type: String,
    pub provider_origin: ProviderOrigin,
    pub max_retries: Option<i32>,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub stream_idle_timeout_seconds: Option<f64>,
    pub priority: i32,
    pub keep_priority_on_conversion: bool,
    pub enable_format_conversion: bool,
    pub is_active: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderRecordPatch {
    pub name: Option<String>,
    pub provider_type: Option<String>,
    pub max_retries: PatchField<i32>,
    pub request_timeout_seconds: PatchField<f64>,
    pub stream_first_byte_timeout_seconds: PatchField<f64>,
    pub stream_idle_timeout_seconds: PatchField<f64>,
    pub priority: Option<i32>,
    pub keep_priority_on_conversion: Option<bool>,
    pub enable_format_conversion: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderEndpointRecordInput {
    pub provider_id: String,
    pub api_format: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub max_retries: Option<i32>,
    pub is_active: bool,
    pub format_acceptance_config: Option<serde_json::Value>,
    pub header_rules: Option<serde_json::Value>,
    pub body_rules: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderEndpointRecordPatch {
    pub api_format: Option<String>,
    pub base_url: Option<String>,
    pub custom_path: PatchField<String>,
    pub max_retries: PatchField<i32>,
    pub is_active: Option<bool>,
    pub format_acceptance_config: PatchField<serde_json::Value>,
    pub header_rules: PatchField<serde_json::Value>,
    pub body_rules: PatchField<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderApiKeyRecordInput {
    pub provider_id: String,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub encrypted_api_key: String,
    pub note: Option<String>,
    pub internal_priority: i32,
    pub global_priority_by_format: BTreeMap<String, i32>,
    pub rpm_limit: Option<i32>,
    pub cache_ttl_minutes: i32,
    pub max_probe_interval_minutes: i32,
    pub time_range_enabled: bool,
    pub time_range_start: Option<String>,
    pub time_range_end: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderApiKeyRecordPatch {
    pub name: Option<String>,
    pub api_formats: Option<Vec<String>>,
    pub allowed_model_ids: Option<Vec<String>>,
    pub encrypted_api_key: Option<String>,
    pub note: PatchField<String>,
    pub internal_priority: Option<i32>,
    pub global_priority_by_format: Option<BTreeMap<String, i32>>,
    pub rpm_limit: PatchField<i32>,
    pub cache_ttl_minutes: Option<i32>,
    pub max_probe_interval_minutes: Option<i32>,
    pub time_range_enabled: Option<bool>,
    pub time_range_start: PatchField<String>,
    pub time_range_end: PatchField<String>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderApiKeySecretRecord {
    pub id: String,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub encrypted_api_key: String,
    pub internal_priority: i32,
    pub global_priority_by_format: BTreeMap<String, i32>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderApiKeyPriorityRecordPatch {
    pub provider_id: String,
    pub key_id: String,
    pub global_priority_by_format: BTreeMap<String, i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderKeyGroupRecordInput {
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub provider_key_members: Vec<ProviderKeyGroupMemberInput>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderKeyGroupRecordPatch {
    pub name: Option<String>,
    pub description: PatchField<String>,
    pub sort_order: Option<i64>,
    pub provider_key_members: PatchField<Vec<ProviderKeyGroupMemberInput>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderModelRecordInput {
    pub provider_id: String,
    pub global_model_id: String,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderModelRecordBatchUpdate {
    pub provider_id: String,
    pub create: Vec<ProviderModelRecordInput>,
    pub delete_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderModelRecordPatch {
    pub is_active: Option<bool>,
    pub config: types::model::PatchField<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderKeyModelMappingRecordInput {
    pub provider_id: String,
    pub key_id: String,
    pub provider_model_id: String,
    pub upstream_model_name: String,
    pub reasoning_effort: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderKeyModelMappingRecordPatch {
    pub upstream_model_name: Option<String>,
    pub reasoning_effort: PatchField<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderKeyModelMappingView {
    pub id: String,
    pub provider_id: String,
    pub key_id: String,
    pub provider_model_id: String,
    pub global_model_id: String,
    pub upstream_model_name: String,
    pub reasoning_effort: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderKeyModelMappingsForProviderRecord {
    pub provider_id: String,
    pub key_id: String,
    pub key_name: String,
    pub is_quick_import_key: bool,
    pub mappings: Vec<ProviderKeyModelMappingView>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderKeyModelMappingsForKeyRecord {
    pub provider_id: String,
    pub key_id: String,
    pub key_name: String,
    pub is_quick_import_key: bool,
    pub mappings: Vec<ProviderKeyModelMappingView>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderCooldownRecordInput {
    pub provider_id: String,
    pub provider_name_snapshot: String,
    pub status_code: i32,
    pub observed_count: i64,
    pub threshold_count: i64,
    pub window_seconds: i64,
    pub cooldown_seconds: i64,
    pub triggered_at: time::OffsetDateTime,
    pub cooldown_until: time::OffsetDateTime,
    pub request_id: String,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub endpoint_id: Option<String>,
    pub endpoint_name_snapshot: Option<String>,
    pub key_id: Option<String>,
    pub key_name_snapshot: Option<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
}

pub type ProviderCooldownEventRecordInput = ProviderCooldownRecordInput;

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderModelCostRecordInput {
    pub provider_id: String,
    pub key_id: String,
    pub provider_model_id: String,
    pub cost_mode: ProviderModelCostMode,
    pub price_per_request: Option<Decimal>,
    pub input_price_per_million: Option<Decimal>,
    pub output_price_per_million: Option<Decimal>,
    pub cache_creation_price_per_million: Option<Decimal>,
    pub cache_read_price_per_million: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportRecordInput {
    pub provider: ProviderRecordInput,
    pub sync_source: Option<ProviderQuickImportSourceRecordInput>,
    pub endpoints: Vec<ProviderQuickImportEndpointRecordInput>,
    pub api_keys: Vec<ProviderQuickImportApiKeyRecordInput>,
    pub model_bindings: Vec<ProviderQuickImportModelRecordInput>,
    pub model_costs: Vec<ProviderQuickImportModelCostRecordInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportAppendRecordInput {
    pub provider_id: String,
    pub source_id: String,
    pub endpoints: Vec<ProviderQuickImportEndpointRecordInput>,
    pub api_keys: Vec<ProviderQuickImportApiKeyRecordInput>,
    pub model_bindings: Vec<ProviderQuickImportModelRecordInput>,
    pub model_costs: Vec<ProviderQuickImportModelCostRecordInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportBoundApiKeyRecordInput {
    pub local_key_id: Option<String>,
    pub input: ProviderQuickImportApiKeyRecordInput,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportBindRecordInput {
    pub provider_id: String,
    pub sync_source: ProviderQuickImportSourceRecordInput,
    pub endpoints: Vec<ProviderQuickImportEndpointRecordInput>,
    pub api_keys: Vec<ProviderQuickImportBoundApiKeyRecordInput>,
    pub model_bindings: Vec<ProviderQuickImportModelRecordInput>,
    pub model_costs: Vec<ProviderQuickImportModelCostRecordInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSourceRecordInput {
    pub source_kind: String,
    pub base_url: String,
    pub encrypted_system_access_token: String,
    pub email: String,
    pub encrypted_password: String,
    pub encrypted_auth_token: String,
    pub encrypted_refresh_token: String,
    pub token_expires_at: Option<time::OffsetDateTime>,
    pub user_id: String,
    pub recharge_multiplier: Decimal,
    pub sync_config: ProviderQuickImportSyncConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportEndpointRecordInput {
    pub api_format: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub max_retries: Option<i32>,
    pub is_active: bool,
    pub format_acceptance_config: Option<serde_json::Value>,
    pub header_rules: Option<serde_json::Value>,
    pub body_rules: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportKeyModelRecordInput {
    pub global_model_id: String,
    pub upstream_model_name: String,
    pub reasoning_effort: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportApiKeyRecordInput {
    pub upstream_token_id: String,
    pub upstream_token_name: String,
    pub upstream_masked_key: String,
    pub upstream_group: Option<String>,
    pub upstream_group_ratio: Decimal,
    pub effective_cost_multiplier: Decimal,
    pub model_mappings: Vec<ProviderQuickImportKeyModelRecordInput>,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub encrypted_api_key: String,
    pub note: Option<String>,
    pub internal_priority: i32,
    pub global_priority_by_format: BTreeMap<String, i32>,
    pub rpm_limit: Option<i32>,
    pub cache_ttl_minutes: i32,
    pub max_probe_interval_minutes: i32,
    pub time_range_enabled: bool,
    pub time_range_start: Option<String>,
    pub time_range_end: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSourceRecord {
    pub id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub source_kind: String,
    pub base_url: String,
    pub encrypted_system_access_token: String,
    pub email: String,
    pub encrypted_password: String,
    pub encrypted_auth_token: String,
    pub encrypted_refresh_token: String,
    pub token_expires_at: Option<time::OffsetDateTime>,
    pub user_id: String,
    pub recharge_multiplier: Decimal,
    pub sync_config: ProviderQuickImportSyncConfig,
    pub last_status: Option<ProviderQuickImportSyncStatus>,
    pub last_error: Option<String>,
    pub last_synced_at: Option<time::OffsetDateTime>,
    pub consecutive_failures: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderQuickImportSourceRecordPatch {
    pub base_url: Option<String>,
    pub encrypted_system_access_token: Option<String>,
    pub email: Option<String>,
    pub encrypted_password: Option<String>,
    pub encrypted_auth_token: Option<String>,
    pub encrypted_refresh_token: Option<String>,
    pub token_expires_at: Option<Option<time::OffsetDateTime>>,
    pub user_id: Option<String>,
    pub recharge_multiplier: Option<Decimal>,
    pub sync_config: Option<ProviderQuickImportSyncConfig>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSyncKeyRecord {
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
    pub model_mappings: Vec<ProviderQuickImportSyncKeyModelRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderQuickImportSyncKeyModelRecord {
    pub provider_model_id: String,
    pub global_model_id: String,
    pub upstream_model_name: String,
    pub reasoning_effort: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportSyncKeyRecordPatch {
    pub key_id: String,
    pub statuses: Vec<ProviderQuickImportSyncStatus>,
    pub upstream_group: Option<Option<String>>,
    pub upstream_group_ratio: Option<Decimal>,
    pub effective_cost_multiplier: Option<Decimal>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportKeyReplacementRecordInput {
    pub provider_id: String,
    pub source_id: String,
    pub key_id: String,
    pub upstream_token_id: String,
    pub upstream_token_name: String,
    pub upstream_masked_key: String,
    pub upstream_group: Option<String>,
    pub upstream_group_ratio: Decimal,
    pub effective_cost_multiplier: Decimal,
    pub model_mappings: Vec<ProviderQuickImportKeyModelRecordInput>,
    pub key_patch: ProviderApiKeyRecordPatch,
    pub model_bindings: Vec<ProviderQuickImportModelRecordInput>,
    pub model_costs: Vec<ProviderQuickImportModelCostRecordInput>,
    pub delete_provider_model_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderQuickImportSyncEventRecordInput {
    pub provider_id: String,
    pub source_id: String,
    pub key_id: Option<String>,
    pub status: ProviderQuickImportSyncStatus,
    pub title: String,
    pub detail: String,
    pub payload: Option<ProviderQuickImportSyncEventPayload>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportModelRecordInput {
    pub global_model_id: String,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportModelCostRecordInput {
    pub upstream_token_id: String,
    pub global_model_id: String,
    pub cost_mode: ProviderModelCostMode,
    pub price_per_request: Option<Decimal>,
    pub input_price_per_million: Option<Decimal>,
    pub output_price_per_million: Option<Decimal>,
    pub cache_creation_price_per_million: Option<Decimal>,
    pub cache_read_price_per_million: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportRecordOutput {
    pub provider: types::provider::Provider,
    pub endpoints: Vec<types::provider::ProviderEndpoint>,
    pub api_keys: Vec<types::provider::ProviderApiKey>,
    pub model_bindings: Vec<types::provider::ProviderModelBinding>,
    pub model_costs: Vec<types::provider::ProviderModelCost>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportAppendRecordOutput {
    pub endpoints: Vec<types::provider::ProviderEndpoint>,
    pub api_keys: Vec<types::provider::ProviderApiKey>,
    pub model_bindings: Vec<types::provider::ProviderModelBinding>,
    pub model_costs: Vec<types::provider::ProviderModelCost>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportBindRecordOutput {
    pub provider: types::provider::Provider,
    pub endpoints: Vec<types::provider::ProviderEndpoint>,
    pub api_keys: Vec<types::provider::ProviderApiKey>,
    pub model_bindings: Vec<types::provider::ProviderModelBinding>,
    pub model_costs: Vec<types::provider::ProviderModelCost>,
    pub created_key_count: usize,
    pub reused_key_count: usize,
    pub deleted_key_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderQuickImportKeyReplacementRecordOutput {
    pub api_key: types::provider::ProviderApiKey,
    pub model_bindings: Vec<types::provider::ProviderModelBinding>,
    pub model_costs: Vec<types::provider::ProviderModelCost>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BillingRuleRecordInput {
    pub global_model_id: Option<String>,
    pub model_id: Option<String>,
    pub name: String,
    pub task_type: String,
    pub expression: String,
    pub variables: Value,
    pub dimension_mappings: Value,
    pub is_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DimensionCollectorRecordInput {
    pub api_format: String,
    pub task_type: String,
    pub dimension_name: String,
    pub source_type: String,
    pub source_path: Option<String>,
    pub value_type: String,
    pub transform_expression: Option<String>,
    pub default_value: Option<String>,
    pub priority: i32,
    pub is_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestCandidateRecordInput {
    pub request_id: String,
    pub token_id: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_name_snapshot: Option<String>,
    pub endpoint_id: Option<String>,
    pub endpoint_name_snapshot: Option<String>,
    pub key_id: Option<String>,
    pub key_name_snapshot: Option<String>,
    pub key_preview_snapshot: Option<String>,
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub needs_conversion: bool,
    pub is_stream: bool,
    pub is_cached: bool,
    pub provider_request_headers: Option<Value>,
    pub provider_request_body: Option<Value>,
    pub provider_response_headers: Option<Value>,
    pub provider_response_body: Option<Value>,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub status: String,
    pub skip_reason: Option<String>,
    pub status_code: Option<i32>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub input_text_tokens: Option<i64>,
    pub input_audio_tokens: Option<i64>,
    pub input_image_tokens: Option<i64>,
    pub output_text_tokens: Option<i64>,
    pub output_audio_tokens: Option<i64>,
    pub output_image_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub cache_creation_5m_input_tokens: Option<i64>,
    pub cache_creation_1h_input_tokens: Option<i64>,
    pub usage_source: Option<String>,
    pub usage_semantic: Option<String>,
    pub upstream_cost: RequestUpstreamCost,
    pub billing: RequestBillingRecordValues,
    pub billing_snapshot: Option<Value>,
    pub latency_ms: Option<i64>,
    pub response_headers_time_ms: Option<i64>,
    pub first_sse_event_time_ms: Option<i64>,
    pub first_output_time_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
    pub started: bool,
    pub finished: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestCandidateRecordPatch {
    pub request_id: String,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub status: String,
    pub skip_reason: Option<String>,
    pub status_code: Option<i32>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub input_text_tokens: Option<i64>,
    pub input_audio_tokens: Option<i64>,
    pub input_image_tokens: Option<i64>,
    pub output_text_tokens: Option<i64>,
    pub output_audio_tokens: Option<i64>,
    pub output_image_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub cache_creation_5m_input_tokens: Option<i64>,
    pub cache_creation_1h_input_tokens: Option<i64>,
    pub usage_source: Option<String>,
    pub usage_semantic: Option<String>,
    pub upstream_cost: RequestUpstreamCostRecordPatch,
    pub billing: RequestBillingRecordValues,
    pub billing_snapshot: PatchField<Value>,
    pub latency_ms: Option<i64>,
    pub response_headers_time_ms: Option<i64>,
    pub first_sse_event_time_ms: Option<i64>,
    pub first_output_time_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
    pub provider_request_headers: PatchField<Value>,
    pub provider_request_body: PatchField<Value>,
    pub provider_response_headers: PatchField<Value>,
    pub provider_response_body: PatchField<Value>,
    pub finished: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestRecordRecordInput {
    pub request_id: String,
    pub token_id: Option<String>,
    pub user_id_snapshot: Option<String>,
    pub username_snapshot: Option<String>,
    pub token_name_snapshot: Option<String>,
    pub token_prefix_snapshot: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: Option<String>,
    pub model_name_snapshot: Option<String>,
    pub provider_id: Option<String>,
    pub provider_name_snapshot: Option<String>,
    pub endpoint_id: Option<String>,
    pub key_id: Option<String>,
    pub provider_key_name_snapshot: Option<String>,
    pub provider_key_preview_snapshot: Option<String>,
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub request_type: String,
    pub is_stream: bool,
    pub has_failover: bool,
    pub has_retry: bool,
    pub status: String,
    pub billing_status: String,
    pub upstream_cost: RequestUpstreamCost,
    pub billing: RequestBillingRecordValues,
    pub billing_snapshot: Option<Value>,
    pub candidate_count: i64,
    pub request_headers: Option<Value>,
    pub request_body: Option<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestRecordRecordPatch {
    pub request_id: String,
    pub provider_id: Option<String>,
    pub provider_name_snapshot: Option<String>,
    pub endpoint_id: Option<String>,
    pub key_id: Option<String>,
    pub provider_key_name_snapshot: Option<String>,
    pub provider_key_preview_snapshot: Option<String>,
    pub provider_api_format: Option<String>,
    pub is_stream: Option<bool>,
    pub has_failover: Option<bool>,
    pub has_retry: Option<bool>,
    pub status: String,
    pub billing_status: String,
    pub client_status_code: PatchField<i32>,
    pub client_error_type: PatchField<String>,
    pub client_error_message: PatchField<String>,
    pub termination_origin: PatchField<String>,
    pub termination_reason: PatchField<String>,
    pub stream_end_reason: PatchField<String>,
    pub prompt_tokens: PatchField<i64>,
    pub completion_tokens: PatchField<i64>,
    pub total_tokens: PatchField<i64>,
    pub cache_creation_input_tokens: PatchField<i64>,
    pub cache_read_input_tokens: PatchField<i64>,
    pub input_text_tokens: PatchField<i64>,
    pub input_audio_tokens: PatchField<i64>,
    pub input_image_tokens: PatchField<i64>,
    pub output_text_tokens: PatchField<i64>,
    pub output_audio_tokens: PatchField<i64>,
    pub output_image_tokens: PatchField<i64>,
    pub reasoning_tokens: PatchField<i64>,
    pub cache_creation_5m_input_tokens: PatchField<i64>,
    pub cache_creation_1h_input_tokens: PatchField<i64>,
    pub usage_source: PatchField<String>,
    pub usage_semantic: PatchField<String>,
    pub upstream_cost: RequestUpstreamCostRecordPatch,
    pub billing: RequestBillingRecordPatch,
    pub billing_snapshot: PatchField<Value>,
    pub response_headers_time_ms: PatchField<i64>,
    pub first_sse_event_time_ms: PatchField<i64>,
    pub first_output_time_ms: PatchField<i64>,
    pub first_byte_time_ms: PatchField<i64>,
    pub total_latency_ms: PatchField<i64>,
    pub client_response_headers: PatchField<Value>,
    pub client_response_body: PatchField<Value>,
    pub started: bool,
    pub finished: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestBillingRecordValues {
    pub service_tier: Option<String>,
    pub cost_currency: Option<String>,
    pub input_cost: Option<Decimal>,
    pub output_cost: Option<Decimal>,
    pub cache_creation_cost: Option<Decimal>,
    pub cache_read_cost: Option<Decimal>,
    pub request_cost: Option<Decimal>,
    pub token_cost: Option<Decimal>,
    pub base_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub billing_multiplier: Option<Decimal>,
    pub input_price_per_million: Option<Decimal>,
    pub output_price_per_million: Option<Decimal>,
    pub cache_creation_price_per_million: Option<Decimal>,
    pub cache_read_price_per_million: Option<Decimal>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestBillingRecordPatch {
    pub service_tier: PatchField<String>,
    pub cost_currency: PatchField<String>,
    pub input_cost: PatchField<Decimal>,
    pub output_cost: PatchField<Decimal>,
    pub cache_creation_cost: PatchField<Decimal>,
    pub cache_read_cost: PatchField<Decimal>,
    pub request_cost: PatchField<Decimal>,
    pub token_cost: PatchField<Decimal>,
    pub base_cost: PatchField<Decimal>,
    pub total_cost: PatchField<Decimal>,
    pub billing_multiplier: PatchField<Decimal>,
    pub input_price_per_million: PatchField<Decimal>,
    pub output_price_per_million: PatchField<Decimal>,
    pub cache_creation_price_per_million: PatchField<Decimal>,
    pub cache_read_price_per_million: PatchField<Decimal>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestUpstreamCostRecordPatch {
    pub upstream_cost_mode: PatchField<ProviderModelCostMode>,
    pub upstream_cost_source: PatchField<ProviderModelCostSource>,
    pub upstream_price_per_request: PatchField<Decimal>,
    pub upstream_input_price_per_million: PatchField<Decimal>,
    pub upstream_output_price_per_million: PatchField<Decimal>,
    pub upstream_cache_creation_price_per_million: PatchField<Decimal>,
    pub upstream_cache_read_price_per_million: PatchField<Decimal>,
    pub upstream_request_cost: PatchField<Decimal>,
    pub upstream_input_cost: PatchField<Decimal>,
    pub upstream_output_cost: PatchField<Decimal>,
    pub upstream_cache_creation_cost: PatchField<Decimal>,
    pub upstream_cache_read_cost: PatchField<Decimal>,
    pub upstream_total_cost: PatchField<Decimal>,
}
