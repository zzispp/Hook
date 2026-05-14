use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const DEFAULT_REQUEST_RECORD_LIMIT: u64 = 20;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RequestRecordListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_request_record_limit")]
    pub limit: u64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub api_format: Option<String>,
    #[serde(default, rename = "type")]
    pub type_filter: Option<String>,
}

impl Default for RequestRecordListRequest {
    fn default() -> Self {
        Self {
            skip: 0,
            limit: default_request_record_limit(),
            search: None,
            status: None,
            model_id: None,
            provider_id: None,
            api_format: None,
            type_filter: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RequestRecordListResponse {
    pub records: Vec<RequestRecord>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ActiveRequestRecordRequest {
    #[serde(default)]
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActiveRequestRecordResponse {
    pub records: Vec<RequestRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RequestRecord {
    pub request_id: String,
    pub created_at: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub token_id: Option<String>,
    pub token_name: Option<String>,
    pub token_prefix: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: Option<String>,
    pub model_name: Option<String>,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub provider_key_name: Option<String>,
    pub provider_key_preview: Option<String>,
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub request_type: String,
    pub is_stream: bool,
    pub has_failover: bool,
    pub has_retry: bool,
    pub status: String,
    pub billing_status: String,
    pub client_status_code: Option<i32>,
    pub client_error_type: Option<String>,
    pub client_error_message: Option<String>,
    pub termination_origin: Option<String>,
    pub termination_reason: Option<String>,
    pub stream_end_reason: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub token_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub base_cost: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub billing_multiplier: Decimal,
    pub cost_currency: String,
    pub first_byte_time_ms: Option<i64>,
    pub total_latency_ms: Option<i64>,
    pub candidate_count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RequestRecordDetail {
    pub record: RequestRecord,
    pub candidates: Vec<RequestCandidateDetail>,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<serde_json::Value>,
    pub client_response_headers: Option<serde_json::Value>,
    pub client_response_body: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RequestCandidateDetail {
    pub id: String,
    pub request_id: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub endpoint_id: Option<String>,
    pub endpoint_name: Option<String>,
    pub key_id: Option<String>,
    pub key_name: Option<String>,
    pub key_preview: Option<String>,
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
    pub provider_request_headers: Option<serde_json::Value>,
    pub provider_request_body: Option<serde_json::Value>,
    pub provider_response_headers: Option<serde_json::Value>,
    pub provider_response_body: Option<serde_json::Value>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

fn default_request_record_limit() -> u64 {
    DEFAULT_REQUEST_RECORD_LIMIT
}
