use std::{error::Error, time::Duration};

use storage::{Database, provider::ProviderStore, setting::SettingStore};
use time::OffsetDateTime;

const SECONDS_PER_DAY: u64 = 86_400;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(SECONDS_PER_DAY);

type CleanupResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn spawn_request_record_cleanup(database: Database) {
    tokio::spawn(cleanup_loop(database));
}

async fn cleanup_loop(database: Database) {
    run_and_log(database.clone()).await;
    loop {
        tokio::time::sleep(CLEANUP_INTERVAL).await;
        run_and_log(database.clone()).await;
    }
}

async fn run_and_log(database: Database) {
    match run_cleanup(database).await {
        Ok(report) => hook_tracing::info_with_fields!(
            "request record cleanup completed",
            deleted_records = report.deleted_records,
            cleared_payloads = report.cleared_payloads,
        ),
        Err(error) => hook_tracing::error("request record cleanup failed", error.as_ref()),
    }
}

async fn run_cleanup(database: Database) -> CleanupResult<CleanupReport> {
    let settings = SettingStore::new(database.clone()).get_system_settings().await?;
    let now = OffsetDateTime::now_utc();
    let record_cutoff = now - time::Duration::days(settings.request_record_retention_days);
    let payload_cutoff = now - time::Duration::days(settings.request_record_payload_retention_days);
    let store = ProviderStore::new(database);
    let deleted_records = store.delete_request_records_before(record_cutoff).await?;
    let cleared_payloads = store.clear_request_record_payloads_before(payload_cutoff).await?;
    Ok(CleanupReport {
        deleted_records,
        cleared_payloads,
    })
}

struct CleanupReport {
    deleted_records: u64,
    cleared_payloads: u64,
}
