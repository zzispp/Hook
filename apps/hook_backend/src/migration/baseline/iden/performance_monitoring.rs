use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum PerformanceMonitoringSnapshots {
    Table,
    Id,
    BucketGranularity,
    BucketStartedAt,
    BucketEndedAt,
    Metrics,
    CreatedAt,
    UpdatedAt,
}
