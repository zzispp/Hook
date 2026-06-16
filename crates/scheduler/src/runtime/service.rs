use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use futures_util::StreamExt;
use storage::{
    Database,
    scheduler::{ScheduledTaskRecord, ScheduledTaskRecordPatch, SchedulerStore},
};
use tokio::sync::{Mutex, mpsc};
use tokio_util::time::{DelayQueue, delay_queue::Key};
use types::{
    pagination::Page,
    scheduler::{ScheduledTask, ScheduledTaskRun, ScheduledTaskRunListRequest, ScheduledTaskUpdate},
};

use crate::runtime::{
    ScheduledTaskLifecycle, SchedulerError, SchedulerRegistry, SchedulerResult,
    query::{list_runs, list_tasks, next_runtime_config, task_definition},
    worker::dispatch_task,
};

const MIN_INTERVAL_SECONDS: i64 = 1;

#[async_trait]
pub trait SchedulerUseCase: Send + Sync + 'static {
    async fn list_tasks(&self) -> SchedulerResult<Vec<ScheduledTask>>;
    async fn update_task(&self, code: &str, input: ScheduledTaskUpdate) -> SchedulerResult<ScheduledTask>;
    async fn list_runs(&self, request: ScheduledTaskRunListRequest) -> SchedulerResult<Page<ScheduledTaskRun>>;
}

#[derive(Clone)]
pub struct SchedulerHandle {
    sender: mpsc::Sender<RuntimeCommand>,
}

impl SchedulerHandle {
    pub async fn reload(&self, code: String) -> SchedulerResult<()> {
        self.sender
            .send(RuntimeCommand::Reload(code))
            .await
            .map_err(|error| SchedulerError::Infrastructure(format!("scheduler reload send failed: {error}")))
    }
}

#[derive(Clone)]
pub struct SchedulerService {
    store: SchedulerStore,
    registry: Arc<SchedulerRegistry>,
    handle: SchedulerHandle,
}

impl SchedulerService {
    pub fn new(store: SchedulerStore, registry: Arc<SchedulerRegistry>, handle: SchedulerHandle) -> Self {
        Self { store, registry, handle }
    }
}

#[async_trait]
impl SchedulerUseCase for SchedulerService {
    async fn list_tasks(&self) -> SchedulerResult<Vec<ScheduledTask>> {
        list_tasks(&self.store, &self.registry).await
    }

    async fn update_task(&self, code: &str, input: ScheduledTaskUpdate) -> SchedulerResult<ScheduledTask> {
        let definition = task_definition(&self.registry, code)?;
        validate_update(&input)?;
        let current = self.store.task_record(code).await?.ok_or_else(|| SchedulerError::NotFound(code.to_owned()))?;
        let next_config = next_runtime_config(&current, input.config.clone())?;
        let task = task_runner(&self.registry, code)?;
        task.validate_config(&next_config)?;
        let record = self
            .store
            .update_task(
                &definition,
                ScheduledTaskRecordPatch {
                    enabled: input.enabled,
                    interval_seconds: input.interval_seconds,
                    config: Some(next_config),
                },
            )
            .await?;
        self.handle.reload(code.to_owned()).await?;
        record.response(&definition).map_err(|error| SchedulerError::Infrastructure(error.to_string()))
    }

    async fn list_runs(&self, request: ScheduledTaskRunListRequest) -> SchedulerResult<Page<ScheduledTaskRun>> {
        list_runs(&self.store, request).await
    }
}

pub struct SchedulerRuntime {
    database: Database,
    registry: Arc<SchedulerRegistry>,
    store: SchedulerStore,
    commands: mpsc::Receiver<RuntimeCommand>,
    queue: DelayQueue<String>,
    keys: HashMap<String, Key>,
    running: Arc<Mutex<HashSet<String>>>,
}

#[derive(Clone)]
pub(crate) enum RuntimeCommand {
    Reload(String),
}

impl SchedulerRuntime {
    pub fn spawn(database: Database, registry: Arc<SchedulerRegistry>) -> SchedulerResult<SchedulerHandle> {
        let store = SchedulerStore::new(database.clone());
        let (sender, receiver) = mpsc::channel(128);
        let mut runtime = Self {
            database,
            registry,
            store,
            commands: receiver,
            queue: DelayQueue::new(),
            keys: HashMap::new(),
            running: Arc::new(Mutex::new(HashSet::new())),
        };
        tokio::spawn(async move {
            if let Err(error) = runtime.run().await {
                hook_tracing::error("scheduler runtime failed", &error);
            }
        });
        Ok(SchedulerHandle { sender })
    }

    async fn run(&mut self) -> SchedulerResult<()> {
        let definitions = self.registry.definitions();
        self.store.ensure_registered_tasks(&definitions).await?;
        self.schedule_all().await?;

        loop {
            tokio::select! {
                Some(command) = self.commands.recv() => {
                    self.handle_command(command).await?;
                }
                maybe_expired = self.queue.next() => {
                    let Some(expired) = maybe_expired else {
                        return Err(SchedulerError::Infrastructure("scheduler delay queue stopped".into()));
                    };
                    let task_code = expired.into_inner();
                    self.keys.remove(&task_code);
                    self.dispatch(task_code).await?;
                }
            }
        }
    }

    async fn schedule_all(&mut self) -> SchedulerResult<()> {
        for code in self.registry.task_codes() {
            self.reschedule(&code).await?;
        }
        Ok(())
    }

    async fn handle_command(&mut self, command: RuntimeCommand) -> SchedulerResult<()> {
        match command {
            RuntimeCommand::Reload(code) => self.reschedule(&code).await,
        }
    }

    async fn reschedule(&mut self, code: &str) -> SchedulerResult<()> {
        if let Some(key) = self.keys.remove(code) {
            let _ = self.queue.remove(&key);
        }
        let Some(record) = self.store.task_record(code).await? else {
            return Ok(());
        };
        if !record.enabled {
            return Ok(());
        }
        let delay = next_attempt_delay(&record, time::OffsetDateTime::now_utc())?;
        let key = self.queue.insert(code.to_owned(), delay);
        self.keys.insert(code.to_owned(), key);
        Ok(())
    }

    async fn dispatch(&mut self, code: String) -> SchedulerResult<()> {
        let task = task_runner(&self.registry, &code)?;
        dispatch_task(&self.store, self.running.clone(), &code, task, self.database.clone()).await?;
        self.reschedule(&code).await?;
        Ok(())
    }
}

fn validate_update(input: &ScheduledTaskUpdate) -> SchedulerResult<()> {
    if input.enabled.is_none() && input.interval_seconds.is_none() && input.config.is_none() {
        return Err(SchedulerError::InvalidInput("update payload is empty".into()));
    }
    if input.interval_seconds.is_some_and(|value| value < MIN_INTERVAL_SECONDS) {
        return Err(SchedulerError::InvalidInput(format!(
            "interval_seconds must be greater than or equal to {MIN_INTERVAL_SECONDS}"
        )));
    }
    Ok(())
}

fn next_attempt_delay(record: &ScheduledTaskRecord, now: time::OffsetDateTime) -> SchedulerResult<Duration> {
    let next_attempt_at = next_attempt_at(record);
    if next_attempt_at <= now {
        return Ok(Duration::ZERO);
    }
    (next_attempt_at - now)
        .try_into()
        .map_err(|_| SchedulerError::Infrastructure("scheduler next attempt delay overflowed".into()))
}

fn next_attempt_at(record: &ScheduledTaskRecord) -> time::OffsetDateTime {
    record
        .locked_until
        .map_or(record.next_run_at, |locked_until| locked_until.max(record.next_run_at))
}

fn task_runner(registry: &Arc<SchedulerRegistry>, code: &str) -> SchedulerResult<Arc<dyn ScheduledTaskLifecycle>> {
    registry.task(code).ok_or_else(|| SchedulerError::NotFound(code.to_owned()))
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;
