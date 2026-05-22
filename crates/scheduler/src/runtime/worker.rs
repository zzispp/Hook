use std::{
    collections::HashSet,
    sync::Arc,
    time::Duration,
};

use storage::{
    Database,
    scheduler::{ScheduledTaskRunRecordInput, ScheduledTaskRunRecordPatch, ScheduledTaskRunStatus, SchedulerStore},
};
use tokio::sync::Mutex;

use crate::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerResult, TaskResult};

pub async fn dispatch_task(
    store: &SchedulerStore,
    running: Arc<Mutex<HashSet<String>>>,
    code: &str,
    task: Arc<dyn ScheduledTaskLifecycle>,
    database: Database,
    config: serde_json::Value,
) -> SchedulerResult<()> {
    if is_running(running.clone(), code).await {
        record_skipped(store, code).await?;
        return Ok(());
    }
    spawn_task(store.clone(), running, code.to_owned(), task, database, config);
    Ok(())
}

pub fn interval_delay(interval_seconds: i64) -> SchedulerResult<Duration> {
    let seconds = u64::try_from(interval_seconds)
        .map_err(|_| SchedulerError::InvalidInput("interval_seconds must be greater than 0".into()))?;
    Ok(Duration::from_secs(seconds))
}

pub async fn is_running(running: Arc<Mutex<HashSet<String>>>, code: &str) -> bool {
    running.lock().await.contains(code)
}

pub async fn mark_running(running: Arc<Mutex<HashSet<String>>>, code: &str, active: bool) {
    let mut guard = running.lock().await;
    if active {
        guard.insert(code.to_owned());
        return;
    }
    guard.remove(code);
}

async fn record_skipped(store: &SchedulerStore, code: &str) -> SchedulerResult<()> {
    let started_at = time::OffsetDateTime::now_utc();
    let run_id = store
        .start_run(ScheduledTaskRunRecordInput {
            task_code: code.to_owned(),
            status: ScheduledTaskRunStatus::Running,
            started_at,
            message: Some("task execution skipped because previous run is still active".into()),
            error: None,
        })
        .await?;
    let finished_at = time::OffsetDateTime::now_utc();
    let duration_ms = duration_millis(started_at, finished_at)?;
    let patch = ScheduledTaskRunRecordPatch {
        status: ScheduledTaskRunStatus::SkippedRunning,
        finished_at,
        duration_ms,
        message: Some("task execution skipped because previous run is still active".into()),
        error: None,
    };
    store.finish_run(&run_id, patch.clone()).await?;
    store.update_task_last_run(code, patch).await?;
    Ok(())
}

fn spawn_task(
    store: SchedulerStore,
    running: Arc<Mutex<HashSet<String>>>,
    code: String,
    task: Arc<dyn ScheduledTaskLifecycle>,
    database: Database,
    config: serde_json::Value,
) {
    tokio::spawn(async move {
        execute_task_run(store, running, code, task, database, config).await;
    });
}

async fn execute_task_run(
    store: SchedulerStore,
    running: Arc<Mutex<HashSet<String>>>,
    code: String,
    task: Arc<dyn ScheduledTaskLifecycle>,
    database: Database,
    config: serde_json::Value,
) {
    mark_running(running.clone(), &code, true).await;
    let started_at = time::OffsetDateTime::now_utc();
    if let Err(error) = store.mark_task_started(&code, started_at).await {
        mark_running(running, &code, false).await;
        hook_tracing::error("scheduler mark task started failed", &error);
        return;
    }
    let run_id = match start_task_run(&store, &code, started_at).await {
        Ok(id) => id,
        Err(error) => {
            mark_running(running, &code, false).await;
            hook_tracing::error("scheduler start run failed", &error);
            return;
        }
    };
    let patch = finish_patch(task.run(ScheduleTaskContext { database }, config).await, started_at);
    if let Err(error) = store.finish_run(&run_id, patch.clone()).await {
        hook_tracing::error("scheduler finish run failed", &error);
    }
    if let Err(error) = store.update_task_last_run(&code, patch).await {
        hook_tracing::error("scheduler update last run failed", &error);
    }
    mark_running(running, &code, false).await;
}

async fn start_task_run(store: &SchedulerStore, code: &str, started_at: time::OffsetDateTime) -> SchedulerResult<String> {
    store
        .start_run(ScheduledTaskRunRecordInput {
            task_code: code.to_owned(),
            status: ScheduledTaskRunStatus::Running,
            started_at,
            message: None,
            error: None,
        })
        .await
        .map_err(Into::into)
}

fn finish_patch(result: TaskResult, started_at: time::OffsetDateTime) -> ScheduledTaskRunRecordPatch {
    let finished_at = time::OffsetDateTime::now_utc();
    let duration_ms = duration_millis(started_at, finished_at).expect("task runtime duration must fit into i64");
    match result {
        Ok(message) => ScheduledTaskRunRecordPatch {
            status: ScheduledTaskRunStatus::Succeeded,
            finished_at,
            duration_ms,
            message,
            error: None,
        },
        Err(error) => ScheduledTaskRunRecordPatch {
            status: ScheduledTaskRunStatus::Failed,
            finished_at,
            duration_ms,
            message: None,
            error: Some(error.to_string()),
        },
    }
}

fn duration_millis(started_at: time::OffsetDateTime, finished_at: time::OffsetDateTime) -> SchedulerResult<i64> {
    (finished_at - started_at)
        .whole_milliseconds()
        .try_into()
        .map_err(|_| SchedulerError::Infrastructure("task duration overflowed i64 milliseconds".into()))
}
