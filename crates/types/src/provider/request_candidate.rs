use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const DEFAULT_REQUEST_CANDIDATE_LIMIT: u64 = 100;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RequestCandidate {
    pub id: String,
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
    pub service_tier: Option<String>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub input_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub output_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_creation_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_read_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub request_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub input_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub output_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_creation_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub cache_read_price_per_million: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub token_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub base_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub total_cost: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub billing_multiplier: Option<Decimal>,
    pub cost_currency: Option<String>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct RequestCandidateListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_request_candidate_limit")]
    pub limit: u64,
    #[serde(default)]
    pub request_id: Option<String>,
}

fn default_request_candidate_limit() -> u64 {
    DEFAULT_REQUEST_CANDIDATE_LIMIT
}
