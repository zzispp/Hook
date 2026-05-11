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
    pub status_code: Option<i32>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
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
