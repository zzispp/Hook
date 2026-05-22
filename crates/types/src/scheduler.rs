use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduledTaskConfigValueType {
    Integer,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScheduledTaskConfigField {
    pub key: String,
    pub label_key: String,
    pub value_type: ScheduledTaskConfigValueType,
    pub required: bool,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub unit_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ScheduledTaskDefinition {
    pub code: String,
    pub name_key: String,
    pub description_key: String,
    pub default_enabled: bool,
    pub default_interval_seconds: i64,
    pub default_config: serde_json::Value,
    pub config_schema: Vec<ScheduledTaskConfigField>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ScheduledTask {
    pub code: String,
    pub name_key: String,
    pub description_key: String,
    pub enabled: bool,
    pub interval_seconds: i64,
    pub config: serde_json::Value,
    pub config_schema: Vec<ScheduledTaskConfigField>,
    pub last_started_at: Option<String>,
    pub last_finished_at: Option<String>,
    pub last_status: Option<String>,
    pub last_duration_ms: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ScheduledTaskUpdate {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub interval_seconds: Option<i64>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScheduledTaskRun {
    pub id: String,
    pub task_code: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ScheduledTaskRunListRequest {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub task_code: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}
