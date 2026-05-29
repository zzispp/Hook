use serde::{Deserialize, Serialize};

const DEFAULT_MODEL_STATUS_RUN_PAGE_SIZE: u64 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatusValue {
    Operational,
    Degraded,
    Failed,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum ModelStatusRangePreset {
    #[serde(rename = "today")]
    Today,
    #[serde(rename = "yesterday")]
    Yesterday,
    #[serde(rename = "last7days", alias = "last7_days", alias = "last7d")]
    Last7Days,
    #[default]
    #[serde(rename = "last30days", alias = "last30_days", alias = "last30d")]
    Last30Days,
    #[serde(rename = "last90days", alias = "last90_days", alias = "last90d")]
    Last90Days,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ModelStatusListRequest {
    #[serde(default)]
    pub preset: ModelStatusRangePreset,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub api_format: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelStatusCheckCreate {
    pub name: String,
    pub global_model_id: String,
    pub api_format: String,
    pub api_token_id: String,
    pub interval_seconds: i64,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelStatusCheckBatchCreateRequest {
    pub name_prefix: String,
    pub global_model_ids: Vec<String>,
    pub api_formats: Vec<String>,
    pub api_token_id: String,
    pub interval_seconds: i64,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckBatchCreateResponse {
    pub success_count: u64,
    pub failed: Vec<ModelStatusCheckBatchCreateFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckBatchCreateFailure {
    pub global_model_id: String,
    pub api_format: String,
    pub error: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ModelStatusCheckUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub global_model_id: Option<String>,
    #[serde(default)]
    pub api_format: Option<String>,
    #[serde(default)]
    pub api_token_id: Option<String>,
    #[serde(default)]
    pub interval_seconds: Option<i64>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelStatusCheckBatchDeleteRequest {
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckBatchDeleteResponse {
    pub success_count: u64,
    pub failed: Vec<ModelStatusCheckBatchDeleteFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckBatchDeleteFailure {
    pub id: String,
    pub error: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelStatusCheckBatchUpdateRequest {
    pub ids: Vec<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub interval_seconds: Option<i64>,
    #[serde(default)]
    pub name_prefix: Option<String>,
    #[serde(default)]
    pub api_token_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckBatchUpdateResponse {
    pub success_count: u64,
    pub failed: Vec<ModelStatusCheckBatchUpdateFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckBatchUpdateFailure {
    pub id: String,
    pub error: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelStatusRunListRequest {
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_run_page_size")]
    pub page_size: u64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub api_format: Option<String>,
    #[serde(default)]
    pub status: Option<ModelStatusValue>,
}

impl Default for ModelStatusRunListRequest {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: DEFAULT_MODEL_STATUS_RUN_PAGE_SIZE,
            search: None,
            api_format: None,
            status: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckResponse {
    pub id: String,
    pub name: String,
    pub global_model_id: String,
    pub model_name: String,
    pub api_format: String,
    pub api_token_id: String,
    pub api_token_name: String,
    pub interval_seconds: i64,
    pub enabled: bool,
    pub next_due_at: String,
    pub last_status: Option<ModelStatusValue>,
    pub last_checked_at: Option<String>,
    pub last_latency_ms: Option<i64>,
    pub last_message: Option<String>,
    pub availability: ModelStatusAvailability,
    pub timeline: Vec<ModelStatusTimelinePoint>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusRunResponse {
    pub id: String,
    pub check_id: String,
    pub check_name: String,
    pub global_model_id: String,
    pub model_name: String,
    pub api_format: String,
    pub api_token_id: String,
    pub api_token_name: String,
    pub status: ModelStatusValue,
    pub latency_ms: Option<i64>,
    pub status_code: Option<i32>,
    pub message: Option<String>,
    pub checked_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusRunListResponse {
    pub items: Vec<ModelStatusRunResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusCheckListResponse {
    pub checks: Vec<ModelStatusCheckResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusAvailability {
    pub total_checks: i64,
    pub available_checks: i64,
    pub availability_pct: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelStatusTimelinePoint {
    pub status: ModelStatusValue,
    pub latency_ms: Option<i64>,
    pub status_code: Option<i32>,
    pub message: Option<String>,
    pub checked_at: String,
}

impl ModelStatusValue {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Operational => "operational",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
            Self::Error => "error",
        }
    }
}

fn default_run_page_size() -> u64 {
    DEFAULT_MODEL_STATUS_RUN_PAGE_SIZE
}
