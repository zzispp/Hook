use std::sync::Arc;

use scheduler::runtime::{DurationExt, ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerResult, TaskConfigValue, TaskResult};
use storage::{
    performance_monitoring::{PerformanceMonitoringStore, SnapshotAggregationWindow},
    scheduler::task_definition,
};
use types::{performance_monitoring::SnapshotGranularity, scheduler::ScheduledTaskDefinition};

use super::{integer_config, integer_fields, performance_collector_error, storage_error, validate_empty_config, validate_positive_integer};
use crate::performance_monitoring_os::PerformanceOsCollector;

#[derive(Clone)]
pub(super) struct PerformanceMonitoringSnapshotTask {
    pub(super) os_collector: Arc<PerformanceOsCollector>,
}

#[derive(Clone, Copy)]
pub(super) struct PerformanceMonitoringCleanupTask;

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for PerformanceMonitoringSnapshotTask {
    fn definition(&self) -> ScheduledTaskDefinition {
        task_definition(
            "performance_monitoring_snapshot",
            "scheduledTasks.definitions.performanceMonitoringSnapshot.name",
            "scheduledTasks.definitions.performanceMonitoringSnapshot.description",
            60,
            60,
            serde_json::json!({}),
            Vec::new(),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_empty_config(config)
    }

    async fn run(&self, ctx: ScheduleTaskContext, _config: TaskConfigValue) -> TaskResult {
        let windows = aggregation_windows(time::OffsetDateTime::now_utc());
        let count = windows.len();
        let system = self.os_collector.clone().snapshot().await.map_err(performance_collector_error)?;
        let store = PerformanceMonitoringStore::new(ctx.database);
        for window in windows {
            store.aggregate_window_with_system(window, system.clone()).await.map_err(storage_error)?;
        }
        Ok(Some(format!("buckets={count}")))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for PerformanceMonitoringCleanupTask {
    fn definition(&self) -> ScheduledTaskDefinition {
        task_definition(
            "performance_monitoring_cleanup",
            "scheduledTasks.definitions.performanceMonitoringCleanup.name",
            "scheduledTasks.definitions.performanceMonitoringCleanup.description",
            24_i64.hours(),
            24_i64.hours(),
            serde_json::json!({
                "retention_days": 30
            }),
            integer_fields(&[("retention_days", "scheduledTasks.config.performanceMonitoringCleanup.retentionDays", 1)]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "retention_days", 1)
    }

    async fn run(&self, ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let retention_days = integer_config(&config, "retention_days")?;
        let cutoff = time::OffsetDateTime::now_utc() - time::Duration::days(retention_days);
        let deleted = PerformanceMonitoringStore::new(ctx.database)
            .delete_snapshots_before(cutoff)
            .await
            .map_err(storage_error)?;
        Ok(Some(format!("deleted_snapshots={deleted}")))
    }
}

fn aggregation_windows(now: time::OffsetDateTime) -> Vec<SnapshotAggregationWindow> {
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

fn window(granularity: SnapshotGranularity, started_at: time::OffsetDateTime) -> SnapshotAggregationWindow {
    SnapshotAggregationWindow {
        granularity,
        started_at,
        ended_at: started_at + time::Duration::seconds(granularity.bucket_seconds()),
    }
}

fn floor_minute(value: time::OffsetDateTime) -> time::OffsetDateTime {
    value.replace_second(0).unwrap().replace_nanosecond(0).unwrap()
}

fn floor_hour(value: time::OffsetDateTime) -> time::OffsetDateTime {
    floor_minute(value).replace_minute(0).unwrap()
}

fn floor_day(value: time::OffsetDateTime) -> time::OffsetDateTime {
    floor_hour(value).replace_hour(0).unwrap()
}
