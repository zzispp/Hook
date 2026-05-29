use types::api_token::ApiToken;

#[derive(Clone, Debug)]
pub struct ModelStatusDueRecord {
    pub check_id: String,
    pub model_name: String,
    pub api_format: String,
    pub interval_seconds: i64,
    pub token: ApiToken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelStatusRunRecordInput {
    pub check_id: String,
    pub status: ModelStatusRunStatusValue,
    pub latency_ms: Option<i64>,
    pub status_code: Option<i32>,
    pub message: Option<String>,
    pub checked_at: time::OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModelStatusRetentionReport {
    pub deleted_runs: u64,
    pub deleted_hourly_stats: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelStatusRunStatusValue {
    Operational,
    Degraded,
    Failed,
    Error,
}

impl ModelStatusRunStatusValue {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Operational => "operational",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
            Self::Error => "error",
        }
    }
}
