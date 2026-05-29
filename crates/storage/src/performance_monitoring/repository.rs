use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use types::performance_monitoring::{
    EffectiveTimeRange, MAX_SERIES_POINTS, PerformanceMonitoringAnalyticsRequest, PerformanceMonitoringAnalyticsResponse,
    PerformanceMonitoringOverviewResponse, PerformanceMonitoringRange, PerformanceSnapshotMetrics, PerformanceSnapshotPoint, SnapshotDataStatus,
    SnapshotGranularity,
};

use crate::{Database, StorageResult};

use super::{
    query::{self, range_plan},
    record::snapshots,
    types::{PerformanceSnapshotInput, SnapshotAggregationWindow, SnapshotQueryPlan, SystemMetricsSnapshot},
};

#[derive(Clone)]
pub struct PerformanceMonitoringStore {
    database: Database,
}

impl PerformanceMonitoringStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn upsert_snapshot(&self, input: PerformanceSnapshotInput) -> StorageResult<()> {
        query::upsert_snapshot(self, input).await
    }

    pub async fn overview(&self, range: PerformanceMonitoringRange, now: time::OffsetDateTime) -> StorageResult<PerformanceMonitoringOverviewResponse> {
        self.overview_with_system(range, now, SystemMetricsSnapshot::default()).await
    }

    pub async fn analytics(
        &self,
        request: PerformanceMonitoringAnalyticsRequest,
        now: time::OffsetDateTime,
    ) -> StorageResult<PerformanceMonitoringAnalyticsResponse> {
        super::analytics::analytics(self, request, now).await
    }

    pub async fn overview_with_system(
        &self,
        range: PerformanceMonitoringRange,
        now: time::OffsetDateTime,
        system: SystemMetricsSnapshot,
    ) -> StorageResult<PerformanceMonitoringOverviewResponse> {
        let plan = range_plan(range, now);
        let records = query::list_snapshots(self, &plan).await?;
        let live_window = live_tail_window(&plan, records.last().map(|record| record.bucket_ended_at));
        let series = records.into_iter().map(snapshot_point).collect::<StorageResult<Vec<_>>>()?;
        let series = self.append_live_tail(series, live_window, system).await?;
        Ok(overview_response(range, plan, series))
    }

    pub async fn aggregate_point(
        &self,
        window: super::SnapshotAggregationWindow,
        system: super::SystemMetricsSnapshot,
    ) -> StorageResult<PerformanceSnapshotPoint> {
        let metrics = super::aggregation::aggregate_window_point(self, window.clone(), system).await?;
        Ok(window_point(window, metrics))
    }

    pub async fn delete_snapshots_before(&self, cutoff: time::OffsetDateTime) -> StorageResult<u64> {
        super::retention::delete_snapshots_before(self, cutoff).await
    }

    pub async fn aggregate_window_with_system(&self, window: super::SnapshotAggregationWindow, system: super::SystemMetricsSnapshot) -> StorageResult<()> {
        super::aggregation::aggregate_window_with_system(self, window, system).await
    }

    async fn append_live_tail(
        &self,
        mut series: Vec<PerformanceSnapshotPoint>,
        window: Option<SnapshotAggregationWindow>,
        system: SystemMetricsSnapshot,
    ) -> StorageResult<Vec<PerformanceSnapshotPoint>> {
        let Some(window) = window else {
            return Ok(series);
        };
        let point = self.aggregate_point(window, system).await?;
        push_live_point(&mut series, point);
        Ok(series)
    }

    pub(crate) fn connection(&self) -> &DatabaseConnection {
        self.database.connection()
    }

    pub(crate) fn next_id(&self) -> String {
        self.database.next_id()
    }

    pub(crate) async fn find_snapshot(&self, granularity: SnapshotGranularity, started_at: time::OffsetDateTime) -> StorageResult<Option<snapshots::Model>> {
        snapshots::Entity::find()
            .filter(snapshots::Column::BucketGranularity.eq(granularity.as_str()))
            .filter(snapshots::Column::BucketStartedAt.eq(started_at))
            .one(self.connection())
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn insert_snapshot(&self, input: PerformanceSnapshotInput, metrics: String) -> StorageResult<()> {
        let now = time::OffsetDateTime::now_utc();
        let active = snapshots::ActiveModel {
            id: Set(self.next_id()),
            bucket_granularity: Set(input.bucket_granularity.as_str().into()),
            bucket_started_at: Set(input.bucket_started_at),
            bucket_ended_at: Set(input.bucket_ended_at),
            metrics: Set(metrics),
            created_at: Set(now),
            updated_at: Set(now),
        };
        active.insert(self.connection()).await?;
        Ok(())
    }

    pub(crate) async fn update_snapshot(&self, record: snapshots::Model, input: PerformanceSnapshotInput, metrics: String) -> StorageResult<()> {
        let mut active: snapshots::ActiveModel = record.into();
        active.bucket_ended_at = Set(input.bucket_ended_at);
        active.metrics = Set(metrics);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.connection()).await?;
        Ok(())
    }
}

fn overview_response(
    range: PerformanceMonitoringRange,
    plan: SnapshotQueryPlan,
    series: Vec<PerformanceSnapshotPoint>,
) -> PerformanceMonitoringOverviewResponse {
    PerformanceMonitoringOverviewResponse {
        range,
        effective_range: effective_range(&plan, &series),
        bucket_granularity: plan.granularity,
        max_series_points: MAX_SERIES_POINTS,
        status: data_status(&series),
        series,
    }
}

fn effective_range(plan: &SnapshotQueryPlan, series: &[PerformanceSnapshotPoint]) -> EffectiveTimeRange {
    if plan.effective_all
        && let (Some(first), Some(last)) = (series.first(), series.last())
    {
        return EffectiveTimeRange {
            started_at: first.bucket_started_at.clone(),
            ended_at: last.bucket_ended_at.clone(),
        };
    }
    EffectiveTimeRange {
        started_at: query::format_timestamp(plan.started_at),
        ended_at: query::format_timestamp(plan.ended_at),
    }
}

fn data_status(series: &[PerformanceSnapshotPoint]) -> SnapshotDataStatus {
    if series.is_empty() {
        return SnapshotDataStatus::EmptySnapshot;
    }
    SnapshotDataStatus::Ready
}

fn live_tail_window(plan: &SnapshotQueryPlan, last_bucket_ended_at: Option<time::OffsetDateTime>) -> Option<SnapshotAggregationWindow> {
    let started_at = last_bucket_ended_at.unwrap_or(plan.started_at).max(plan.started_at);
    if started_at >= plan.ended_at {
        return None;
    }
    Some(SnapshotAggregationWindow {
        granularity: plan.granularity,
        started_at,
        ended_at: plan.ended_at,
    })
}

fn push_live_point(series: &mut Vec<PerformanceSnapshotPoint>, point: PerformanceSnapshotPoint) {
    if !should_append_live_point(series, &point) {
        return;
    }
    series.push(point);
    if series.len() > MAX_SERIES_POINTS {
        series.drain(0..series.len() - MAX_SERIES_POINTS);
    }
}

fn should_append_live_point(series: &[PerformanceSnapshotPoint], point: &PerformanceSnapshotPoint) -> bool {
    if series.last().is_some_and(|last| last.bucket_ended_at >= point.bucket_ended_at) {
        return false;
    }
    !series.is_empty() || point.metrics.core.request_count > 0
}

fn snapshot_point(record: snapshots::Model) -> StorageResult<PerformanceSnapshotPoint> {
    Ok(PerformanceSnapshotPoint {
        bucket_started_at: query::format_timestamp(record.bucket_started_at),
        bucket_ended_at: query::format_timestamp(record.bucket_ended_at),
        metrics: serde_json::from_str(&record.metrics)?,
    })
}

fn window_point(window: super::SnapshotAggregationWindow, metrics: PerformanceSnapshotMetrics) -> PerformanceSnapshotPoint {
    PerformanceSnapshotPoint {
        bucket_started_at: query::format_timestamp(window.started_at),
        bucket_ended_at: query::format_timestamp(window.ended_at),
        metrics,
    }
}

#[cfg(test)]
mod tests {
    use types::performance_monitoring::{CoreRequestMetrics, PerformanceSnapshotMetrics, SnapshotGranularity};

    use super::{PerformanceSnapshotPoint, live_tail_window, push_live_point};
    use crate::performance_monitoring::SnapshotQueryPlan;

    #[test]
    fn live_tail_window_starts_after_last_persisted_bucket() {
        let plan = plan(0, 180);

        let window = live_tail_window(&plan, Some(ts(120))).unwrap();

        assert_eq!(window.started_at, ts(120));
        assert_eq!(window.ended_at, ts(180));
    }

    #[test]
    fn live_tail_window_uses_plan_start_without_persisted_bucket() {
        let plan = plan(0, 180);

        let window = live_tail_window(&plan, None).unwrap();

        assert_eq!(window.started_at, ts(0));
        assert_eq!(window.ended_at, ts(180));
    }

    #[test]
    fn live_tail_window_skips_closed_range() {
        let plan = plan(0, 180);

        assert!(live_tail_window(&plan, Some(ts(180))).is_none());
    }

    #[test]
    fn push_live_point_keeps_non_empty_current_data_without_snapshots() {
        let mut series = Vec::new();

        push_live_point(&mut series, point(0, 180, 3));

        assert_eq!(series.len(), 1);
        assert_eq!(series[0].metrics.core.request_count, 3);
    }

    #[test]
    fn push_live_point_skips_empty_live_only_series() {
        let mut series = Vec::new();

        push_live_point(&mut series, point(0, 180, 0));

        assert!(series.is_empty());
    }

    fn plan(started_at: i64, ended_at: i64) -> SnapshotQueryPlan {
        SnapshotQueryPlan {
            granularity: SnapshotGranularity::Minute,
            started_at: ts(started_at),
            ended_at: ts(ended_at),
            effective_all: false,
        }
    }

    fn point(started_at: i64, ended_at: i64, request_count: i64) -> PerformanceSnapshotPoint {
        PerformanceSnapshotPoint {
            bucket_started_at: super::query::format_timestamp(ts(started_at)),
            bucket_ended_at: super::query::format_timestamp(ts(ended_at)),
            metrics: PerformanceSnapshotMetrics {
                core: CoreRequestMetrics {
                    request_count,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    fn ts(seconds: i64) -> time::OffsetDateTime {
        time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
    }
}
