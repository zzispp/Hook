use std::{collections::HashSet, sync::Arc};

use sea_orm::{DbBackend, FromQueryResult, Statement, TransactionTrait};
use storage::{
    Database,
    scheduler::{ScheduledTaskClaim, ScheduledTaskRunRecordInput, ScheduledTaskRunRecordPatch, ScheduledTaskRunStatus, SchedulerStore},
};
use tokio::sync::Mutex;

use crate::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerResult, TaskResult};

const LOCAL_RUNNING_MESSAGE: &str = "task execution skipped because previous run is still active";
const DATABASE_RUNNING_MESSAGE: &str = "task execution skipped because another scheduler instance is still active";
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub async fn dispatch_task(
    store: &SchedulerStore,
    running: Arc<Mutex<HashSet<String>>>,
    code: &str,
    task: Arc<dyn ScheduledTaskLifecycle>,
    database: Database,
) -> SchedulerResult<()> {
    let Some(claim) = store.claim_due_task(code, time::OffsetDateTime::now_utc()).await? else {
        return Ok(());
    };
    if is_running(running.clone(), code).await {
        record_claimed_skipped(store, &claim, LOCAL_RUNNING_MESSAGE).await?;
        return Ok(());
    }
    spawn_task(store.clone(), running, task, database, claim);
    Ok(())
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

async fn record_claimed_skipped(store: &SchedulerStore, claim: &ScheduledTaskClaim, message: &str) -> SchedulerResult<()> {
    let run_id = store
        .start_run(ScheduledTaskRunRecordInput {
            task_code: claim.record.code.clone(),
            status: ScheduledTaskRunStatus::Running,
            started_at: claim.started_at,
            message: Some(message.into()),
            error: None,
        })
        .await?;
    let patch = skipped_patch(claim.started_at, message)?;
    store.finish_run(&run_id, patch.clone()).await?;
    finish_claim(store, claim, patch).await?;
    Ok(())
}

fn spawn_task(
    store: SchedulerStore,
    running: Arc<Mutex<HashSet<String>>>,
    task: Arc<dyn ScheduledTaskLifecycle>,
    database: Database,
    claim: ScheduledTaskClaim,
) {
    tokio::spawn(async move {
        execute_task_run(store, running, task, database, claim).await;
    });
}

async fn execute_task_run(
    store: SchedulerStore,
    running: Arc<Mutex<HashSet<String>>>,
    task: Arc<dyn ScheduledTaskLifecycle>,
    database: Database,
    claim: ScheduledTaskClaim,
) {
    let code = claim.record.code.clone();
    mark_running(running.clone(), &code, true).await;
    if let Err(error) = execute_claimed_task_run(&store, task, database, claim).await {
        hook_tracing::error("scheduler execute claimed task failed", &error);
    }
    mark_running(running, &code, false).await;
}

async fn execute_claimed_task_run(
    store: &SchedulerStore,
    task: Arc<dyn ScheduledTaskLifecycle>,
    database: Database,
    claim: ScheduledTaskClaim,
) -> SchedulerResult<()> {
    let config = match claim.record.runtime_config() {
        Ok(config) => config,
        Err(error) => {
            record_claimed_failed(store, &claim, format!("scheduler config decode failed: {error}")).await?;
            return Ok(());
        }
    };
    let code = claim.record.code.as_str();
    let lock = match acquire_task_lock(&database, code).await {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            record_claimed_skipped(store, &claim, DATABASE_RUNNING_MESSAGE).await?;
            return Ok(());
        }
        Err(error) => {
            record_claimed_failed(store, &claim, format!("scheduler startup failed: {error}")).await?;
            return Ok(());
        }
    };
    let run_id = match start_claimed_task_run(store, &claim, None).await {
        Ok(id) => id,
        Err(error) => {
            let finish_result = finish_claim_after_start_failure(store, &claim, error).await;
            commit_task_lock(lock).await;
            finish_result?;
            return Ok(());
        }
    };
    let patch = finish_patch(task.run(ScheduleTaskContext { database }, config).await, claim.started_at);
    if let Err(error) = store.finish_run(&run_id, patch.clone()).await {
        hook_tracing::error("scheduler finish run failed", &error);
    }
    let finish_result = finish_claim(store, &claim, patch).await;
    commit_task_lock(lock).await;
    finish_result?;
    Ok(())
}

async fn acquire_task_lock(database: &Database, code: &str) -> SchedulerResult<Option<sea_orm::DatabaseTransaction>> {
    let tx = database.connection().begin().await.map_err(db_error)?;
    if try_advisory_xact_lock(&tx, code).await? {
        return Ok(Some(tx));
    }
    tx.rollback().await.map_err(db_error)?;
    Ok(None)
}

async fn try_advisory_xact_lock(tx: &sea_orm::DatabaseTransaction, code: &str) -> SchedulerResult<bool> {
    let row = AdvisoryLockRow::find_by_statement(lock_statement(code))
        .one(tx)
        .await
        .map_err(db_error)?
        .ok_or_else(|| SchedulerError::Infrastructure("scheduler advisory lock query returned no rows".into()))?;
    Ok(row.acquired)
}

fn db_error(error: sea_orm::DbErr) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}

fn lock_statement(code: &str) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT pg_try_advisory_xact_lock($1) AS acquired",
        vec![advisory_lock_key(code).into()],
    )
}

fn advisory_lock_key(code: &str) -> i64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in code.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i64
}

#[derive(Debug, FromQueryResult)]
struct AdvisoryLockRow {
    acquired: bool,
}

async fn start_claimed_task_run(store: &SchedulerStore, claim: &ScheduledTaskClaim, message: Option<String>) -> SchedulerResult<String> {
    store
        .start_run(ScheduledTaskRunRecordInput {
            task_code: claim.record.code.clone(),
            status: ScheduledTaskRunStatus::Running,
            started_at: claim.started_at,
            message,
            error: None,
        })
        .await
        .map_err(Into::into)
}

async fn record_claimed_failed(store: &SchedulerStore, claim: &ScheduledTaskClaim, error: String) -> SchedulerResult<()> {
    let run_id = start_claimed_task_run(store, claim, None).await?;
    let patch = failed_patch(claim.started_at, error)?;
    store.finish_run(&run_id, patch.clone()).await?;
    finish_claim(store, claim, patch).await?;
    Ok(())
}

async fn finish_claim_after_start_failure(store: &SchedulerStore, claim: &ScheduledTaskClaim, error: SchedulerError) -> SchedulerResult<()> {
    let patch = failed_patch(claim.started_at, format!("scheduler start run failed: {error}"))?;
    finish_claim(store, claim, patch).await
}

fn skipped_patch(started_at: time::OffsetDateTime, message: &str) -> SchedulerResult<ScheduledTaskRunRecordPatch> {
    let finished_at = time::OffsetDateTime::now_utc();
    Ok(ScheduledTaskRunRecordPatch {
        status: ScheduledTaskRunStatus::SkippedRunning,
        finished_at,
        duration_ms: duration_millis(started_at, finished_at)?,
        message: Some(message.into()),
        error: None,
    })
}

fn failed_patch(started_at: time::OffsetDateTime, error: String) -> SchedulerResult<ScheduledTaskRunRecordPatch> {
    let finished_at = time::OffsetDateTime::now_utc();
    Ok(ScheduledTaskRunRecordPatch {
        status: ScheduledTaskRunStatus::Failed,
        finished_at,
        duration_ms: duration_millis(started_at, finished_at)?,
        message: None,
        error: Some(error),
    })
}

async fn finish_claim(store: &SchedulerStore, claim: &ScheduledTaskClaim, patch: ScheduledTaskRunRecordPatch) -> SchedulerResult<()> {
    let updated = store.finish_claimed_task_run(&claim.record.code, &claim.lock_owner, patch).await?;
    if !updated {
        let task_code = claim.record.code.as_str();
        let lock_owner = claim.lock_owner.as_str();
        hook_tracing::warn_with_fields!(
            "scheduler claim finish ignored because lease owner changed",
            task_code = task_code,
            lock_owner = lock_owner
        );
    }
    Ok(())
}

async fn commit_task_lock(lock: sea_orm::DatabaseTransaction) {
    if let Err(error) = lock.commit().await {
        hook_tracing::error("scheduler advisory lock transaction commit failed", &error);
    }
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

#[cfg(test)]
#[path = "worker_tests.rs"]
mod worker_tests;
