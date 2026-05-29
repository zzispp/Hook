use sea_orm_migration::prelude::*;

#[derive(Clone, Copy, DeriveIden)]
pub(in crate::migration::baseline) enum ModelStatusChecks {
    Table,
    Id,
    Name,
    GlobalModelId,
    ApiFormat,
    ApiTokenId,
    IntervalSeconds,
    Enabled,
    NextDueAt,
    LockedUntil,
    LastStatus,
    LastCheckedAt,
    LastLatencyMs,
    LastMessage,
    CreatedAt,
    UpdatedAt,
}

#[derive(Clone, Copy, DeriveIden)]
pub(in crate::migration::baseline) enum ModelStatusCheckRuns {
    Table,
    Id,
    CheckId,
    Status,
    LatencyMs,
    StatusCode,
    Message,
    CheckedAt,
    CreatedAt,
}

#[derive(Clone, Copy, DeriveIden)]
pub(in crate::migration::baseline) enum ModelStatusCheckHourlyStats {
    Table,
    Id,
    CheckId,
    BucketStartedAt,
    TotalCount,
    AvailableCount,
    DegradedCount,
    FailedCount,
    ErrorCount,
    LatencySumMs,
    CreatedAt,
    UpdatedAt,
}
