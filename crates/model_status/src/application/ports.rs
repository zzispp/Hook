use async_trait::async_trait;
use types::{api_token::ApiToken, model_status::*};

use super::ModelStatusResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelStatusRunRecord {
    pub check_id: String,
    pub status: ModelStatusRunStatus,
    pub latency_ms: Option<i64>,
    pub status_code: Option<i32>,
    pub message: Option<String>,
    pub checked_at: time::OffsetDateTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelStatusRunStatus {
    Operational,
    Degraded,
    Failed,
    Error,
}

#[derive(Clone, Debug)]
pub struct ModelStatusProbeInput {
    pub check_id: String,
    pub model_name: String,
    pub api_format: String,
    pub interval_seconds: i64,
    pub token: ApiToken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelStatusProbeOutput {
    pub status: ModelStatusRunStatus,
    pub latency_ms: Option<i64>,
    pub status_code: Option<i32>,
    pub message: Option<String>,
}

#[async_trait]
pub trait ModelStatusRepository: Send + Sync + 'static {
    async fn list_public(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse>;
    async fn list_admin(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse>;
    async fn create_check(&self, input: ModelStatusCheckCreate) -> ModelStatusResult<ModelStatusCheckResponse>;
    async fn batch_create_checks(&self, input: ModelStatusCheckBatchCreateRequest) -> ModelStatusResult<ModelStatusCheckBatchCreateResponse>;
    async fn update_check(&self, id: &str, input: ModelStatusCheckUpdate) -> ModelStatusResult<ModelStatusCheckResponse>;
    async fn delete_check(&self, id: &str) -> ModelStatusResult<()>;
    async fn batch_update_checks(&self, input: ModelStatusCheckBatchUpdateRequest) -> ModelStatusResult<ModelStatusCheckBatchUpdateResponse>;
    async fn list_runs(&self, request: ModelStatusRunListRequest) -> ModelStatusResult<ModelStatusRunListResponse>;
    async fn due_checks(&self, limit: u64, now: time::OffsetDateTime) -> ModelStatusResult<Vec<ModelStatusProbeInput>>;
    async fn record_run(&self, record: ModelStatusRunRecord, interval_seconds: i64) -> ModelStatusResult<()>;
    async fn token_has_checks(&self, token_id: &str) -> ModelStatusResult<bool>;
}

#[async_trait]
pub trait ModelStatusTokenCatalog: Send + Sync + 'static {
    async fn independent_token(&self, id: &str) -> ModelStatusResult<Option<ApiToken>>;
}

#[async_trait]
pub trait ModelStatusProbe: Send + Sync + 'static {
    async fn probe(&self, input: ModelStatusProbeInput) -> ModelStatusProbeOutput;
}

#[async_trait]
pub trait ModelStatusUseCase: Send + Sync + 'static {
    async fn list_public(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse>;
    async fn list_admin(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse>;
    async fn create_check(&self, input: ModelStatusCheckCreate) -> ModelStatusResult<ModelStatusCheckResponse>;
    async fn batch_create_checks(&self, input: ModelStatusCheckBatchCreateRequest) -> ModelStatusResult<ModelStatusCheckBatchCreateResponse>;
    async fn update_check(&self, id: &str, input: ModelStatusCheckUpdate) -> ModelStatusResult<ModelStatusCheckResponse>;
    async fn delete_check(&self, id: &str) -> ModelStatusResult<()>;
    async fn batch_delete_checks(&self, ids: Vec<String>) -> ModelStatusResult<ModelStatusCheckBatchDeleteResponse>;
    async fn batch_update_checks(&self, input: ModelStatusCheckBatchUpdateRequest) -> ModelStatusResult<ModelStatusCheckBatchUpdateResponse>;
    async fn list_runs(&self, request: ModelStatusRunListRequest) -> ModelStatusResult<ModelStatusRunListResponse>;
    async fn run_due_checks(&self, limit: u64, concurrency: usize) -> ModelStatusResult<u64>;
    async fn token_has_checks(&self, token_id: &str) -> ModelStatusResult<bool>;
}

impl ModelStatusRunStatus {
    pub fn as_value(self) -> ModelStatusValue {
        match self {
            Self::Operational => ModelStatusValue::Operational,
            Self::Degraded => ModelStatusValue::Degraded,
            Self::Failed => ModelStatusValue::Failed,
            Self::Error => ModelStatusValue::Error,
        }
    }
}
