use std::{error::Error, sync::Arc, time::Duration};

use storage::{
    Database,
    performance_monitoring::{PerformanceMonitoringStore, SnapshotAggregationWindow},
    setting::SettingStore,
};
use time::OffsetDateTime;
use types::performance_monitoring::SnapshotGranularity;

use crate::performance_monitoring_os::PerformanceOsCollector;

const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(60);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(86_400);

type WorkerResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn spawn_performance_monitoring_workers(database: Database, os_collector: Arc<PerformanceOsCollector>) {
    tokio::spawn(snapshot_loop(database.clone(), os_collector));
    tokio::spawn(cleanup_loop(database));
}

async fn snapshot_loop(database: Database, os_collector: Arc<PerformanceOsCollector>) {
    run_snapshot_and_log(database.clone(), os_collector.clone()).await;
    loop {
        tokio::time::sleep(SNAPSHOT_INTERVAL).await;
        run_snapshot_and_log(database.clone(), os_collector.clone()).await;
    }
}

async fn cleanup_loop(database: Database) {
    run_cleanup_and_log(database.clone()).await;
    loop {
        tokio::time::sleep(CLEANUP_INTERVAL).await;
        run_cleanup_and_log(database.clone()).await;
    }
}

async fn run_snapshot_and_log(database: Database, os_collector: Arc<PerformanceOsCollector>) {
    match run_snapshot(database, os_collector).await {
        Ok(count) => hook_tracing::info_with_fields!("performance monitoring snapshot completed", buckets = count),
        Err(error) => hook_tracing::error("performance monitoring snapshot failed", error.as_ref()),
    }
}

async fn run_cleanup_and_log(database: Database) {
    match run_cleanup(database).await {
        Ok(deleted) => hook_tracing::info_with_fields!("performance monitoring cleanup completed", deleted_snapshots = deleted),
        Err(error) => hook_tracing::error("performance monitoring cleanup failed", error.as_ref()),
    }
}

async fn run_snapshot(database: Database, os_collector: Arc<PerformanceOsCollector>) -> WorkerResult<usize> {
    let now = OffsetDateTime::now_utc();
    let store = PerformanceMonitoringStore::new(database);
    let windows = aggregation_windows(now);
    let count = windows.len();
    let system = os_collector.clone().snapshot().await?;
    for window in windows {
        store.aggregate_window_with_system(window, system.clone()).await?;
    }
    Ok(count)
}

async fn run_cleanup(database: Database) -> WorkerResult<u64> {
    let settings = SettingStore::new(database.clone()).get_system_settings().await?;
    let cutoff = OffsetDateTime::now_utc() - time::Duration::days(settings.performance_monitoring_retention_days);
    PerformanceMonitoringStore::new(database)
        .delete_snapshots_before(cutoff)
        .await
        .map_err(Into::into)
}

fn aggregation_windows(now: OffsetDateTime) -> Vec<SnapshotAggregationWindow> {
    let minute_start = floor_minute(now) - time::Duration::minutes(1);
    let mut windows = vec![window(SnapshotGranularity::Minute, minute_start)];
    if minute_start.minute() == 59 {
        windows.push(window(SnapshotGranularity::Hour, floor_hour(now) - time::Duration::hours(1)));
    }
    if minute_start.hour() == 23 && minute_start.minute() == 59 {
        windows.push(window(SnapshotGranularity::Day, floor_day(now) - time::Duration::days(1)));
    }
    windows
}

fn window(granularity: SnapshotGranularity, started_at: OffsetDateTime) -> SnapshotAggregationWindow {
    SnapshotAggregationWindow {
        granularity,
        started_at,
        ended_at: started_at + time::Duration::seconds(granularity.bucket_seconds()),
    }
}

fn floor_minute(value: OffsetDateTime) -> OffsetDateTime {
    value.replace_second(0).unwrap().replace_nanosecond(0).unwrap()
}

fn floor_hour(value: OffsetDateTime) -> OffsetDateTime {
    floor_minute(value).replace_minute(0).unwrap()
}

fn floor_day(value: OffsetDateTime) -> OffsetDateTime {
    floor_hour(value).replace_hour(0).unwrap()
}

#[cfg(test)]
mod tests {
    use types::performance_monitoring::SnapshotGranularity;

    use super::aggregation_windows;

    #[test]
    fn snapshot_worker_generates_minute_only_inside_hour() {
        let now = time::macros::datetime!(2026-05-15 08:17:12 UTC);
        let windows = aggregation_windows(now);

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].granularity, SnapshotGranularity::Minute);
        assert_eq!(windows[0].started_at, time::macros::datetime!(2026-05-15 08:16:00 UTC));
    }

    #[test]
    fn snapshot_worker_generates_hour_on_hour_boundary() {
        let now = time::macros::datetime!(2026-05-15 09:00:03 UTC);
        let windows = aggregation_windows(now);

        assert_eq!(windows.len(), 2);
        assert_eq!(windows[1].granularity, SnapshotGranularity::Hour);
        assert_eq!(windows[1].started_at, time::macros::datetime!(2026-05-15 08:00:00 UTC));
    }

    #[test]
    fn snapshot_worker_generates_day_on_midnight_boundary() {
        let now = time::macros::datetime!(2026-05-16 00:00:03 UTC);
        let windows = aggregation_windows(now);

        assert_eq!(windows.len(), 3);
        assert_eq!(windows[2].granularity, SnapshotGranularity::Day);
        assert_eq!(windows[2].started_at, time::macros::datetime!(2026-05-15 00:00:00 UTC));
    }
}
