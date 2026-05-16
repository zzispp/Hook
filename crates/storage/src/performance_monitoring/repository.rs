use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use types::performance_monitoring::{
    EffectiveTimeRange, MAX_SERIES_POINTS, PerformanceMonitoringOverviewResponse, PerformanceMonitoringRange, PerformanceSnapshotPoint, SnapshotDataStatus,
    SnapshotGranularity,
};

use crate::{Database, StorageResult};

use super::{
    query::{self, range_plan},
    record::snapshots,
    types::{PerformanceSnapshotInput, SnapshotQueryPlan},
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
        let plan = range_plan(range, now);
        let records = query::list_snapshots(self, &plan).await?;
        let series = records.into_iter().map(snapshot_point).collect::<StorageResult<Vec<_>>>()?;
        Ok(overview_response(range, plan, series))
    }

    pub async fn latest_snapshot(&self) -> StorageResult<Option<PerformanceSnapshotPoint>> {
        query::latest_snapshot(self).await?.map(snapshot_point).transpose()
    }

    pub async fn delete_snapshots_before(&self, cutoff: time::OffsetDateTime) -> StorageResult<u64> {
        super::retention::delete_snapshots_before(self, cutoff).await
    }

    pub async fn aggregate_window_with_system(&self, window: super::SnapshotAggregationWindow, system: super::SystemMetricsSnapshot) -> StorageResult<()> {
        super::aggregation::aggregate_window_with_system(self, window, system).await
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

fn snapshot_point(record: snapshots::Model) -> StorageResult<PerformanceSnapshotPoint> {
    Ok(PerformanceSnapshotPoint {
        bucket_started_at: query::format_timestamp(record.bucket_started_at),
        bucket_ended_at: query::format_timestamp(record.bucket_ended_at),
        metrics: serde_json::from_str(&record.metrics)?,
    })
}
