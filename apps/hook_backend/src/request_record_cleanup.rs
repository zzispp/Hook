use std::{error::Error, time::Duration};

use storage::{Database, provider::ProviderStore, setting::SettingStore};
use time::OffsetDateTime;
use types::system_setting::SystemSettings;

const SECONDS_PER_HOUR: u64 = 3_600;
const FALLBACK_RETRY_INTERVAL: Duration = Duration::from_secs(SECONDS_PER_HOUR);

type CleanupResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn spawn_request_record_cleanup(database: Database) {
    tokio::spawn(cleanup_loop(database));
}

async fn cleanup_loop(database: Database) {
    loop {
        let interval = match run_and_log(database.clone()).await {
            Ok(value) => value,
            Err(error) => {
                hook_tracing::error("request record cleanup failed", error.as_ref());
                FALLBACK_RETRY_INTERVAL
            }
        };
        tokio::time::sleep(interval).await;
    }
}

async fn run_and_log(database: Database) -> CleanupResult<Duration> {
    let settings = SettingStore::new(database.clone()).get_system_settings().await?;
    let interval = cleanup_interval(settings.request_record_cleanup_interval_hours);
    if !settings.request_record_cleanup_enabled {
        hook_tracing::info_with_fields!("request record cleanup skipped", reason = "auto cleanup disabled");
        return Ok(interval);
    }
    let report = run_cleanup(database, &settings).await?;
    hook_tracing::info_with_fields!(
        "request record cleanup completed",
        deleted_records = report.deleted_records,
        compressed_payloads = report.compressed_payloads,
    );
    Ok(interval)
}

async fn run_cleanup(database: Database, settings: &SystemSettings) -> CleanupResult<CleanupReport> {
    let now = OffsetDateTime::now_utc();
    let record_cutoff = now - time::Duration::days(settings.request_record_retention_days);
    let payload_cutoff = now - time::Duration::days(settings.request_record_payload_retention_days);
    let store = ProviderStore::new(database);
    let deleted_records = store.delete_request_records_before(record_cutoff).await?;
    let compressed_payloads = store.compress_request_record_payloads_before(payload_cutoff).await?;
    Ok(CleanupReport {
        deleted_records,
        compressed_payloads,
    })
}

fn cleanup_interval(hours: i64) -> Duration {
    Duration::from_secs(hours as u64 * SECONDS_PER_HOUR)
}

struct CleanupReport {
    deleted_records: u64,
    compressed_payloads: u64,
}
