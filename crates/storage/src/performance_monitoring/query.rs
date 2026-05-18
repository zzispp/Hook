use sea_orm::{ColumnTrait, DbBackend, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Statement, Value};
use time::format_description::well_known::Rfc3339;
use types::performance_monitoring::{MAX_SERIES_POINTS, PerformanceMonitoringRange, SnapshotGranularity};

use crate::{StorageError, StorageResult};

use super::{
    PerformanceMonitoringStore,
    record::snapshots,
    types::{PerformanceSnapshotInput, SnapshotQueryPlan},
};

const REALTIME_MINUTES: i64 = 5;
const TODAY_HOURS: i64 = 24;
const SEVEN_DAYS: i64 = 7;
const THIRTY_DAYS: i64 = 30;

pub(super) async fn upsert_snapshot(store: &PerformanceMonitoringStore, input: PerformanceSnapshotInput) -> StorageResult<()> {
    let metrics = serde_json::to_string(&input.metrics)?;
    if let Some(record) = store.find_snapshot(input.bucket_granularity, input.bucket_started_at).await? {
        return store.update_snapshot(record, input, metrics).await;
    }
    store.insert_snapshot(input, metrics).await
}

pub(super) async fn list_snapshots(store: &PerformanceMonitoringStore, plan: &SnapshotQueryPlan) -> StorageResult<Vec<snapshots::Model>> {
    if plan.effective_all {
        return list_all_snapshots(store, plan).await;
    }
    snapshots::Entity::find()
        .filter(snapshots::Column::BucketGranularity.eq(plan.granularity.as_str()))
        .filter(snapshots::Column::BucketStartedAt.gte(plan.started_at))
        .filter(snapshots::Column::BucketStartedAt.lt(plan.ended_at))
        .order_by_asc(snapshots::Column::BucketStartedAt)
        .limit(MAX_SERIES_POINTS as u64)
        .all(store.connection())
        .await
        .map_err(Into::into)
}

pub fn range_plan(range: PerformanceMonitoringRange, now: time::OffsetDateTime) -> SnapshotQueryPlan {
    match range {
        PerformanceMonitoringRange::Realtime => fixed_plan(SnapshotGranularity::Minute, now - time::Duration::minutes(REALTIME_MINUTES), now),
        PerformanceMonitoringRange::Today => fixed_plan(SnapshotGranularity::Hour, now - time::Duration::hours(TODAY_HOURS), now),
        PerformanceMonitoringRange::SevenDays => fixed_plan(SnapshotGranularity::Hour, now - time::Duration::days(SEVEN_DAYS), now),
        PerformanceMonitoringRange::ThirtyDays => thirty_day_plan(now),
        PerformanceMonitoringRange::All => all_plan(now),
    }
}

pub(crate) fn format_timestamp(value: time::OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("performance monitoring timestamp must format as RFC3339")
}

async fn list_all_snapshots(store: &PerformanceMonitoringStore, plan: &SnapshotQueryPlan) -> StorageResult<Vec<snapshots::Model>> {
    let sql = "WITH ranked AS ( \
            SELECT id, bucket_granularity, bucket_started_at, bucket_ended_at, metrics, created_at, updated_at, \
                ROW_NUMBER() OVER (ORDER BY bucket_started_at ASC)::bigint AS row_index, \
                COUNT(*) OVER ()::bigint AS total_count \
            FROM performance_monitoring_snapshots \
            WHERE bucket_granularity = $1 \
        ), slotted AS ( \
            SELECT *, CASE \
                WHEN total_count <= $2 THEN row_index \
                WHEN row_index = 1 THEN 1::bigint \
                WHEN row_index = total_count THEN $2::bigint \
                ELSE LEAST(($2 - 1)::bigint, GREATEST(2::bigint, \
                    FLOOR(((row_index - 1)::numeric * ($2 - 1)::numeric) / NULLIF((total_count - 1)::numeric, 0))::bigint + 1)) \
                END AS sample_slot \
            FROM ranked \
        ), deduped AS ( \
            SELECT DISTINCT ON (sample_slot) id, bucket_granularity, bucket_started_at, bucket_ended_at, metrics, created_at, updated_at \
            FROM slotted \
            ORDER BY sample_slot, \
                CASE WHEN sample_slot = 1 THEN bucket_started_at END ASC, \
                CASE WHEN sample_slot = $2 THEN bucket_started_at END DESC, \
                bucket_started_at ASC \
        ) \
        SELECT id, bucket_granularity, bucket_started_at, bucket_ended_at, metrics, created_at, updated_at \
        FROM deduped \
        ORDER BY bucket_started_at ASC \
        LIMIT $2";
    let values = vec![Value::from(plan.granularity.as_str().to_owned()), Value::from(MAX_SERIES_POINTS as i64)];
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, values);
    snapshots::Model::find_by_statement(statement)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

fn fixed_plan(granularity: SnapshotGranularity, started_at: time::OffsetDateTime, ended_at: time::OffsetDateTime) -> SnapshotQueryPlan {
    SnapshotQueryPlan {
        granularity,
        started_at,
        ended_at,
        effective_all: false,
    }
}

fn thirty_day_plan(now: time::OffsetDateTime) -> SnapshotQueryPlan {
    let started_at = now - time::Duration::days(THIRTY_DAYS);
    let granularity = safe_granularity(started_at, now, SnapshotGranularity::Hour);
    fixed_plan(granularity, started_at, now)
}

fn all_plan(now: time::OffsetDateTime) -> SnapshotQueryPlan {
    SnapshotQueryPlan {
        granularity: SnapshotGranularity::Day,
        started_at: floor_day(now),
        ended_at: now,
        effective_all: true,
    }
}

fn floor_day(value: time::OffsetDateTime) -> time::OffsetDateTime {
    value
        .replace_hour(0)
        .unwrap()
        .replace_minute(0)
        .unwrap()
        .replace_second(0)
        .unwrap()
        .replace_nanosecond(0)
        .unwrap()
}

fn safe_granularity(started_at: time::OffsetDateTime, ended_at: time::OffsetDateTime, preferred: SnapshotGranularity) -> SnapshotGranularity {
    if estimated_points(started_at, ended_at, preferred) <= MAX_SERIES_POINTS {
        return preferred;
    }
    SnapshotGranularity::Day
}

fn estimated_points(started_at: time::OffsetDateTime, ended_at: time::OffsetDateTime, granularity: SnapshotGranularity) -> usize {
    let seconds = (ended_at - started_at).whole_seconds().max(0);
    let points = seconds / granularity.bucket_seconds();
    usize::try_from(points).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use types::performance_monitoring::PerformanceMonitoringRange;

    use super::{MAX_SERIES_POINTS, SnapshotGranularity, range_plan};

    #[test]
    fn realtime_range_uses_last_five_minute_buckets() {
        let now = time::OffsetDateTime::from_unix_timestamp(600).unwrap();
        let plan = range_plan(PerformanceMonitoringRange::Realtime, now);

        assert_eq!(plan.granularity, SnapshotGranularity::Minute);
        assert_eq!((plan.ended_at - plan.started_at).whole_minutes(), 5);
    }

    #[test]
    fn today_range_uses_twenty_four_hour_buckets() {
        let now = time::OffsetDateTime::from_unix_timestamp(86_400).unwrap();
        let plan = range_plan(PerformanceMonitoringRange::Today, now);

        assert_eq!(plan.granularity, SnapshotGranularity::Hour);
        assert_eq!((plan.ended_at - plan.started_at).whole_hours(), 24);
    }

    #[test]
    fn range_all_uses_day_buckets_without_detail_range() {
        let now = time::OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
        let plan = range_plan(PerformanceMonitoringRange::All, now);

        assert_eq!(plan.granularity, SnapshotGranularity::Day);
        assert!(plan.started_at < plan.ended_at);
        assert!(plan.effective_all);
    }

    #[test]
    fn thirty_day_range_selects_safe_bucket_count() {
        let now = time::OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
        let plan = range_plan(PerformanceMonitoringRange::ThirtyDays, now);
        let seconds = (plan.ended_at - plan.started_at).whole_seconds();
        let point_count = seconds / plan.granularity.bucket_seconds();

        assert!(point_count <= MAX_SERIES_POINTS as i64);
        assert_eq!(plan.granularity, SnapshotGranularity::Hour);
    }
}
