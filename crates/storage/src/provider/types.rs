use types::model::{PatchField, TieredPricingConfig};

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
    pub encrypted_api_key: String,
    pub note: Option<String>,
    pub api_formats: Option<Vec<String>>,
    pub internal_priority: i32,
    pub rpm_limit: Option<i32>,
    pub cache_ttl_minutes: i32,
    pub max_probe_interval_minutes: i32,
    pub time_range_enabled: bool,
    pub time_range_start: Option<String>,
    pub time_range_end: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderModelRecordInput {
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
    pub price_per_request: Option<rust_decimal::Decimal>,
    pub tiered_pricing: Option<TieredPricingConfig>,
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestCandidateRecordInput {
    pub request_id: String,
    pub token_id: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: Option<String>,
    pub provider_id: Option<String>,
    pub endpoint_id: Option<String>,
    pub key_id: Option<String>,
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub needs_conversion: bool,
    pub is_stream: bool,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub status: String,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub started: bool,
    pub finished: bool,
}
