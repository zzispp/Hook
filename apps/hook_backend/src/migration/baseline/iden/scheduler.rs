use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ScheduledTasks {
    Table,
    Code,
    Enabled,
    IntervalSeconds,
    LeaseSeconds,
    Config,
    NextRunAt,
    LockedUntil,
    LockedBy,
    LastStartedAt,
    LastFinishedAt,
    LastStatus,
    LastDurationMs,
    LastError,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ScheduledTaskRuns {
    Table,
    Id,
    TaskCode,
    Status,
    StartedAt,
    FinishedAt,
    DurationMs,
    Message,
    Error,
}
