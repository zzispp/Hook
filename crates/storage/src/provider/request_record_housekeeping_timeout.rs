use std::time::{Duration, Instant};

use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::StorageResult;

use super::request_record_housekeeping::RequestRecordCleanupOptions;

const STATEMENT_HEADROOM_SECONDS: u64 = 1;

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

    pub(super) fn has_statement_headroom(&self, statement_timeout_seconds: i64) -> bool {
        self.remaining()
            .is_some_and(|remaining| remaining > statement_headroom(statement_timeout_seconds))
    }
}

pub(super) async fn apply_timeouts(tx: &sea_orm::DatabaseTransaction, options: &RequestRecordCleanupOptions) -> StorageResult<()> {
    tx.execute_raw(timeout_statement(options)).await?;
    Ok(())
}

fn timeout_statement(options: &RequestRecordCleanupOptions) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT set_config('statement_timeout', $1, true), set_config('lock_timeout', $2, true)",
        vec![
            timeout_value(timeout_duration(options.statement_timeout_seconds)).into(),
            timeout_value(timeout_duration(options.lock_timeout_seconds)).into(),
        ],
    )
}

fn statement_headroom(statement_timeout_seconds: i64) -> Duration {
    timeout_duration(statement_timeout_seconds).saturating_add(Duration::from_secs(STATEMENT_HEADROOM_SECONDS))
}

fn timeout_duration(configured_seconds: i64) -> Duration {
    Duration::from_secs(configured_seconds as u64)
}

fn timeout_value(duration: Duration) -> String {
    format!("{}ms", duration.as_millis().max(1))
}
