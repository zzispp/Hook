use sea_orm_migration::{prelude::*, schema::*};

use super::iden::{ScheduledTaskRuns, ScheduledTasks};

pub(super) fn scheduler_tables() -> Vec<TableCreateStatement> {
    vec![scheduled_tasks_table(), scheduled_task_runs_table()]
}

fn scheduled_tasks_table() -> TableCreateStatement {
    Table::create()
        .table(ScheduledTasks::Table)
        .if_not_exists()
        .col(string_len(ScheduledTasks::Code, 100).primary_key())
        .col(boolean(ScheduledTasks::Enabled).default(true))
        .col(big_integer(ScheduledTasks::IntervalSeconds))
        .col(text(ScheduledTasks::Config))
        .col(timestamp_tz(ScheduledTasks::NextRunAt))
        .col(timestamp_tz_null(ScheduledTasks::LockedUntil))
        .col(string_len_null(ScheduledTasks::LockedBy, 100))
        .col(timestamp_tz_null(ScheduledTasks::LastStartedAt))
        .col(timestamp_tz_null(ScheduledTasks::LastFinishedAt))
        .col(string_len_null(ScheduledTasks::LastStatus, 40))
        .col(big_integer_null(ScheduledTasks::LastDurationMs))
        .col(text_null(ScheduledTasks::LastError))
        .col(timestamp_tz(ScheduledTasks::CreatedAt))
        .col(timestamp_tz(ScheduledTasks::UpdatedAt))
        .to_owned()
}

fn scheduled_task_runs_table() -> TableCreateStatement {
    let mut task_fk = scheduled_task_run_task_fk();
    Table::create()
        .table(ScheduledTaskRuns::Table)
        .if_not_exists()
        .col(string_len(ScheduledTaskRuns::Id, 36).primary_key())
        .col(string_len(ScheduledTaskRuns::TaskCode, 100))
        .col(string_len(ScheduledTaskRuns::Status, 40))
        .col(timestamp_tz(ScheduledTaskRuns::StartedAt))
        .col(timestamp_tz_null(ScheduledTaskRuns::FinishedAt))
        .col(big_integer_null(ScheduledTaskRuns::DurationMs))
        .col(text_null(ScheduledTaskRuns::Message))
        .col(text_null(ScheduledTaskRuns::Error))
        .foreign_key(&mut task_fk)
        .to_owned()
}

fn scheduled_task_run_task_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_scheduled_task_runs_task")
        .from(ScheduledTaskRuns::Table, ScheduledTaskRuns::TaskCode)
        .to(ScheduledTasks::Table, ScheduledTasks::Code)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}

fn timestamp_tz_null<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().null().take()
}
