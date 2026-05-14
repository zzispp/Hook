use std::{error::Error, time::Duration};

use storage::{Database, provider::ProviderStore};
use time::OffsetDateTime;

const SWEEP_INTERVAL: Duration = Duration::from_secs(300);
const STALE_PENDING_TIMEOUT_MINUTES: i64 = 15;
const STALE_STREAMING_TIMEOUT_MINUTES: i64 = 120;

type SweepResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn spawn_request_record_sweep(database: Database) {
    tokio::spawn(sweep_loop(database));
}

async fn sweep_loop(database: Database) {
    run_and_log(database.clone()).await;
    loop {
        tokio::time::sleep(SWEEP_INTERVAL).await;
        run_and_log(database.clone()).await;
    }
}

async fn run_and_log(database: Database) {
    match run_sweep(database).await {
        Ok(report) => hook_tracing::info_with_fields!(
            "request record stale sweep completed",
            pending_records = report.pending_records,
            streaming_records = report.streaming_records,
            failed_candidates = report.failed_candidates,
            skipped_candidates = report.skipped_candidates,
        ),
        Err(error) => hook_tracing::error("request record stale sweep failed", error.as_ref()),
    }
}

async fn run_sweep(database: Database) -> SweepResult<storage::provider::StaleRequestSweepReport> {
    let now = OffsetDateTime::now_utc();
    let pending_cutoff = now - time::Duration::minutes(STALE_PENDING_TIMEOUT_MINUTES);
    let streaming_cutoff = now - time::Duration::minutes(STALE_STREAMING_TIMEOUT_MINUTES);
    ProviderStore::new(database)
        .sweep_stale_request_records(pending_cutoff, streaming_cutoff)
        .await
        .map_err(Into::into)
}
