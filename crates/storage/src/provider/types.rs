use rust_decimal::Decimal;
use serde_json::Value;
use types::model::{PatchField, TieredPricingConfig};
use types::provider::ProviderModelMapping;

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderRecordInput {
    pub name: String,
    pub provider_type: String,
    pub max_retries: Option<i32>,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
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
    pub rpm_limit: PatchField<i32>,
    pub cache_ttl_minutes: Option<i32>,
    pub max_probe_interval_minutes: Option<i32>,
    pub time_range_enabled: Option<bool>,
    pub time_range_start: PatchField<String>,
    pub time_range_end: PatchField<String>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderApiKeySecretRecord {
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub encrypted_api_key: String,
    pub internal_priority: i32,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderModelRecordInput {
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
    pub provider_model_mapping: Option<ProviderModelMapping>,
    pub is_active: bool,
    pub price_per_request: Option<rust_decimal::Decimal>,
    pub tiered_pricing: Option<TieredPricingConfig>,
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderModelRecordPatch {
    pub provider_model_name: Option<String>,
    pub is_active: Option<bool>,
    pub provider_model_mapping: types::model::PatchField<ProviderModelMapping>,
    pub config: types::model::PatchField<serde_json::Value>,
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
    pub billing: RequestBillingRecordValues,
    pub latency_ms: Option<i64>,
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
    pub billing: RequestBillingRecordValues,
    pub latency_ms: Option<i64>,
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
    pub billing: RequestBillingRecordValues,
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
    pub billing: RequestBillingRecordPatch,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaleRequestSweepReport {
    pub pending_records: u64,
    pub streaming_records: u64,
    pub failed_candidates: u64,
    pub skipped_candidates: u64,
}
