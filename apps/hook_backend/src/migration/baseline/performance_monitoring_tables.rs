use sea_orm_migration::{prelude::*, schema::*};

use super::iden::PerformanceMonitoringSnapshots;

pub(super) fn performance_monitoring_tables() -> Vec<TableCreateStatement> {
    vec![snapshots_table()]
}

fn snapshots_table() -> TableCreateStatement {
    Table::create()
        .table(PerformanceMonitoringSnapshots::Table)
        .if_not_exists()
        .col(string_len(PerformanceMonitoringSnapshots::Id, 36).primary_key())
        .col(string_len(PerformanceMonitoringSnapshots::BucketGranularity, 16))
        .col(timestamp_tz(PerformanceMonitoringSnapshots::BucketStartedAt))
        .col(timestamp_tz(PerformanceMonitoringSnapshots::BucketEndedAt))
        .col(text(PerformanceMonitoringSnapshots::Metrics))
        .col(timestamp_tz(PerformanceMonitoringSnapshots::CreatedAt))
        .col(timestamp_tz(PerformanceMonitoringSnapshots::UpdatedAt))
        .to_owned()
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}
