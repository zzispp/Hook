use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use storage::Database;
use types::scheduler::{ScheduledTaskConfigField, ScheduledTaskDefinition};

use crate::runtime::SchedulerResult;

pub type TaskResult = SchedulerResult<Option<String>>;
pub type TaskConfigValue = serde_json::Value;
pub type ScheduledTaskExecution = Pin<Box<dyn Future<Output = TaskResult> + Send + 'static>>;

#[derive(Clone)]
pub struct ScheduleTaskContext {
    pub database: Database,
}

#[async_trait]
pub trait ScheduledTaskLifecycle: Send + Sync + 'static {
    fn definition(&self) -> ScheduledTaskDefinition;

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()>;

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult;
}

pub trait ScheduledTaskFactory: Send + Sync + 'static {
    fn build(&self) -> Arc<dyn ScheduledTaskLifecycle>;
}

impl<T> ScheduledTaskFactory for T
where
    T: ScheduledTaskLifecycle + Clone,
{
    fn build(&self) -> Arc<dyn ScheduledTaskLifecycle> {
        Arc::new(self.clone())
    }
}

pub trait ScheduledTaskDefinitionExt {
    fn with_fields(self, fields: Vec<ScheduledTaskConfigField>) -> Self;
}

impl ScheduledTaskDefinitionExt for ScheduledTaskDefinition {
    fn with_fields(mut self, fields: Vec<ScheduledTaskConfigField>) -> Self {
        self.config_schema = fields;
        self
    }
}

pub trait DurationExt {
    fn minutes(self) -> i64;
    fn hours(self) -> i64;
    fn days(self) -> i64;
}

impl DurationExt for i64 {
    fn minutes(self) -> i64 {
        self * 60
    }

    fn hours(self) -> i64 {
        self * 3_600
    }

    fn days(self) -> i64 {
        self * 86_400
    }
}
