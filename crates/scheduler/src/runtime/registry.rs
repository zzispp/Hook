use std::{collections::HashMap, sync::Arc};

use crate::runtime::{ScheduledTaskFactory, ScheduledTaskLifecycle, SchedulerError, SchedulerResult};

#[derive(Clone)]
pub struct RegisteredTask {
    pub task: Arc<dyn ScheduledTaskLifecycle>,
}

#[derive(Clone, Default)]
pub struct SchedulerRegistry {
    tasks: HashMap<String, RegisteredTask>,
}

impl SchedulerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T>(&mut self, task: T) -> SchedulerResult<()>
    where
        T: ScheduledTaskFactory,
    {
        let task = task.build();
        let definition = task.definition();
        if self.tasks.contains_key(&definition.code) {
            return Err(SchedulerError::InvalidInput(format!("duplicate scheduled task code: {}", definition.code)));
        }
        self.tasks.insert(definition.code.clone(), RegisteredTask { task });
        Ok(())
    }

    pub fn definitions(&self) -> Vec<types::scheduler::ScheduledTaskDefinition> {
        self.tasks.values().map(|registered| registered.task.definition()).collect()
    }

    pub fn task(&self, code: &str) -> Option<Arc<dyn ScheduledTaskLifecycle>> {
        self.tasks.get(code).map(|registered| registered.task.clone())
    }

    pub fn task_codes(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }
}
