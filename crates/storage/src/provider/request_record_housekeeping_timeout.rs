use std::time::{Duration, Instant};

use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::StorageResult;

use super::request_record_housekeeping::RequestRecordCleanupOptions;

#[derive(Debug)]
pub(super) struct CleanupBudget {
    started: Instant,
    max_runtime: Duration,
}

impl CleanupBudget {
    pub(super) fn start(max_runtime: Duration) -> Self {
        Self {
            started: Instant::now(),
            max_runtime,
        }
    }

    pub(super) fn exhausted(&self) -> bool {
        self.started.elapsed() >= self.max_runtime
    }

    pub(super) fn remaining(&self) -> Option<Duration> {
        self.max_runtime.checked_sub(self.started.elapsed())
    }
}

pub(super) async fn apply_timeouts(tx: &sea_orm::DatabaseTransaction, options: &RequestRecordCleanupOptions, budget: &CleanupBudget) -> StorageResult<()> {
    tx.execute_raw(timeout_statement(options, budget)).await?;
    Ok(())
}

fn timeout_statement(options: &RequestRecordCleanupOptions, budget: &CleanupBudget) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT set_config('statement_timeout', $1, true), set_config('lock_timeout', $2, true)",
        vec![
            timeout_value(effective_timeout(options.statement_timeout_seconds, budget)).into(),
            timeout_value(effective_timeout(options.lock_timeout_seconds, budget)).into(),
        ],
    )
}

fn effective_timeout(configured_seconds: i64, budget: &CleanupBudget) -> Duration {
    let configured = Duration::from_secs(configured_seconds as u64);
    budget.remaining().map_or(configured, |remaining| configured.min(remaining))
}

fn timeout_value(duration: Duration) -> String {
    format!("{}ms", duration.as_millis().max(1))
}
