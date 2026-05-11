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
}

impl Default for RequestRecordListRequest {
    fn default() -> Self {
        Self {
            skip: 0,
            limit: default_request_record_limit(),
            search: None,
            status: None,
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
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub request_type: String,
    pub is_stream: bool,
    pub status: String,
    pub billing_status: String,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub total_cost: Decimal,
    pub first_byte_time_ms: Option<i64>,
    pub total_latency_ms: Option<i64>,
    pub candidate_count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RequestRecordDetail {
    pub record: RequestRecord,
    pub candidates: Vec<RequestCandidateDetail>,
    pub request_body: Option<serde_json::Value>,
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
    pub candidate_index: i32,
    pub retry_index: i32,
    pub status: String,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

fn default_request_record_limit() -> u64 {
    DEFAULT_REQUEST_RECORD_LIMIT
}
