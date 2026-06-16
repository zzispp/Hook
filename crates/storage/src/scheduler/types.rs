#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduledTaskRunStatus {
    Running,
    Succeeded,
    Failed,
    SkippedRunning,
}

impl ScheduledTaskRunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::SkippedRunning => "skipped_running",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScheduledTaskRecordPatch {
    pub enabled: Option<bool>,
    pub interval_seconds: Option<i64>,
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScheduledTaskClaim {
    pub record: super::ScheduledTaskRecord,
    pub lock_owner: String,
    pub started_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledTaskRunRecordInput {
    pub task_code: String,
    pub status: ScheduledTaskRunStatus,
    pub started_at: time::OffsetDateTime,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledTaskRunRecordPatch {
    pub status: ScheduledTaskRunStatus,
    pub finished_at: time::OffsetDateTime,
    pub duration_ms: i64,
    pub message: Option<String>,
    pub error: Option<String>,
}
