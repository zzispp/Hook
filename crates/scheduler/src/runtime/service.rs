use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use futures_util::StreamExt;
use storage::{
    Database,
    scheduler::{ScheduledTaskRecordPatch, SchedulerStore},
};
use time::format_description::well_known::Rfc3339;
use tokio::sync::{Mutex, mpsc};
use tokio_util::time::{DelayQueue, delay_queue::Key};
use types::{
    pagination::Page,
    scheduler::{ScheduledTask, ScheduledTaskRun, ScheduledTaskRunListRequest, ScheduledTaskUpdate},
};

use crate::runtime::{
    ScheduledTaskLifecycle, SchedulerError, SchedulerRegistry, SchedulerResult,
    query::{list_runs, list_tasks, next_runtime_config, task_definition},
    worker::{dispatch_task, interval_delay},
};

const MIN_INTERVAL_SECONDS: i64 = 1;

type NextRunMap = Arc<Mutex<HashMap<String, time::OffsetDateTime>>>;

#[async_trait]
pub trait SchedulerUseCase: Send + Sync + 'static {
    async fn list_tasks(&self) -> SchedulerResult<Vec<ScheduledTask>>;
    async fn update_task(&self, code: &str, input: ScheduledTaskUpdate) -> SchedulerResult<ScheduledTask>;
    async fn list_runs(&self, request: ScheduledTaskRunListRequest) -> SchedulerResult<Page<ScheduledTaskRun>>;
}

#[derive(Clone)]
pub struct SchedulerHandle {
    sender: mpsc::Sender<RuntimeCommand>,
    next_runs: NextRunMap,
}

impl SchedulerHandle {
    pub async fn reload(&self, code: String) -> SchedulerResult<()> {
        self.sender
            .send(RuntimeCommand::Reload(code))
            .await
            .map_err(|error| SchedulerError::Infrastructure(format!("scheduler reload send failed: {error}")))
    }

    async fn next_run_snapshot(&self) -> HashMap<String, time::OffsetDateTime> {
        self.next_runs.lock().await.clone()
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
        let tasks = list_tasks(&self.store, &self.registry).await?;
        with_next_run_times(tasks, self.handle.next_run_snapshot().await)
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
        let mut task = record
            .response(&definition)
            .map_err(|error| SchedulerError::Infrastructure(error.to_string()))?;
        task.next_run_at = next_run_from_now(&task)?;
        Ok(task)
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
    next_runs: NextRunMap,
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
        let next_runs = Arc::new(Mutex::new(HashMap::new()));
        let mut runtime = Self {
            database,
            registry,
            store,
            commands: receiver,
            queue: DelayQueue::new(),
            keys: HashMap::new(),
            next_runs: next_runs.clone(),
            running: Arc::new(Mutex::new(HashSet::new())),
        };
        tokio::spawn(async move {
            if let Err(error) = runtime.run().await {
                hook_tracing::error("scheduler runtime failed", &error);
            }
        });
        Ok(SchedulerHandle { sender, next_runs })
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
        self.next_runs.lock().await.remove(code);
        let Some(record) = self.store.task_record(code).await? else {
            return Ok(());
        };
        if !record.enabled {
            return Ok(());
        }
        let delay = interval_delay(record.interval_seconds)?;
        let next_run_at = next_run_after_interval(record.interval_seconds);
        let key = self.queue.insert(code.to_owned(), delay);
        self.keys.insert(code.to_owned(), key);
        self.next_runs.lock().await.insert(code.to_owned(), next_run_at);
        Ok(())
    }

    async fn dispatch(&mut self, code: String) -> SchedulerResult<()> {
        let Some(record) = self.store.task_record(&code).await? else {
            return Ok(());
        };
        if !record.enabled {
            return Ok(());
        }
        let task = task_runner(&self.registry, &code)?;
        let config = record.runtime_config()?;
        dispatch_task(&self.store, self.running.clone(), &code, task, self.database.clone(), config).await?;
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

fn with_next_run_times(mut tasks: Vec<ScheduledTask>, next_runs: HashMap<String, time::OffsetDateTime>) -> SchedulerResult<Vec<ScheduledTask>> {
    for task in &mut tasks {
        task.next_run_at = next_runs.get(&task.code).copied().map(format_next_run_at).transpose()?;
    }
    Ok(tasks)
}

fn next_run_from_now(task: &ScheduledTask) -> SchedulerResult<Option<String>> {
    if !task.enabled {
        return Ok(None);
    }
    format_next_run_at(next_run_after_interval(task.interval_seconds)).map(Some)
}

fn next_run_after_interval(interval_seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc() + time::Duration::seconds(interval_seconds)
}

fn format_next_run_at(value: time::OffsetDateTime) -> SchedulerResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| SchedulerError::Infrastructure(format!("scheduler next_run_at format failed: {error}")))
}

fn task_runner(registry: &Arc<SchedulerRegistry>, code: &str) -> SchedulerResult<Arc<dyn ScheduledTaskLifecycle>> {
    registry.task(code).ok_or_else(|| SchedulerError::NotFound(code.to_owned()))
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;
