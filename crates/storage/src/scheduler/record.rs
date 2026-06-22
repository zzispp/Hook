use time::format_description::well_known::Rfc3339;
use types::scheduler::{ScheduledTask, ScheduledTaskConfigField, ScheduledTaskDefinition, ScheduledTaskRun};

use crate::{StorageError, StorageResult, json};

pub use super::entities::{scheduled_task_runs, scheduled_tasks};

pub type ScheduledTaskRecord = scheduled_tasks::Model;
pub type ScheduledTaskRunRecord = scheduled_task_runs::Model;

impl ScheduledTaskRecord {
    pub fn runtime_config(&self) -> StorageResult<serde_json::Value> {
        json::decode_required(self.config.clone())
    }

    pub fn response(self, definition: &ScheduledTaskDefinition) -> StorageResult<ScheduledTask> {
        Ok(ScheduledTask {
            code: self.code,
            name_key: definition.name_key.clone(),
            description_key: definition.description_key.clone(),
            enabled: self.enabled,
            interval_seconds: self.interval_seconds,
            lease_seconds: self.lease_seconds,
            next_run_at: self.enabled.then_some(self.next_run_at).map(format_timestamp).transpose()?,
            config: json::decode_required(self.config)?,
            config_schema: definition.config_schema.clone(),
            last_started_at: self.last_started_at.map(format_timestamp).transpose()?,
            last_finished_at: self.last_finished_at.map(format_timestamp).transpose()?,
            last_status: self.last_status,
            last_duration_ms: self.last_duration_ms,
            last_error: self.last_error,
            created_at: format_timestamp(self.created_at)?,
            updated_at: format_timestamp(self.updated_at)?,
        })
    }
}

impl ScheduledTaskRunRecord {
    pub fn response(self) -> StorageResult<ScheduledTaskRun> {
        Ok(ScheduledTaskRun {
            id: self.id,
            task_code: self.task_code,
            status: self.status,
            started_at: format_timestamp(self.started_at)?,
            finished_at: self.finished_at.map(format_timestamp).transpose()?,
            duration_ms: self.duration_ms,
            message: self.message,
            error: self.error,
        })
    }
}

pub fn task_definition(
    code: impl Into<String>,
    name_key: impl Into<String>,
    description_key: impl Into<String>,
    default_interval_seconds: i64,
    default_lease_seconds: i64,
    default_config: serde_json::Value,
    config_schema: Vec<ScheduledTaskConfigField>,
) -> ScheduledTaskDefinition {
    ScheduledTaskDefinition {
        code: code.into(),
        name_key: name_key.into(),
        description_key: description_key.into(),
        default_enabled: true,
        default_interval_seconds,
        default_lease_seconds,
        default_config,
        config_schema,
    }
}

fn format_timestamp(value: time::OffsetDateTime) -> StorageResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| StorageError::Database(format!("scheduler timestamp format failed: {error}")))
}
