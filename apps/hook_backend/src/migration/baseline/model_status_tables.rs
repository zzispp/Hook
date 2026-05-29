use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

pub(super) fn model_status_tables() -> Vec<TableCreateStatement> {
    vec![checks_table(), runs_table(), hourly_stats_table()]
}

fn checks_table() -> TableCreateStatement {
    let mut model_fk = ForeignKey::create();
    model_fk
        .name("fk_model_status_checks_global_model")
        .from(ModelStatusChecks::Table, ModelStatusChecks::GlobalModelId)
        .to(GlobalModels::Table, GlobalModels::Id);
    let mut token_fk = ForeignKey::create();
    token_fk
        .name("fk_model_status_checks_api_token")
        .from(ModelStatusChecks::Table, ModelStatusChecks::ApiTokenId)
        .to(ApiTokens::Table, ApiTokens::Id);
    Table::create()
        .table(ModelStatusChecks::Table)
        .if_not_exists()
        .col(string_len(ModelStatusChecks::Id, 36).primary_key())
        .col(string_len(ModelStatusChecks::Name, 100))
        .col(string_len(ModelStatusChecks::GlobalModelId, 36))
        .col(string_len(ModelStatusChecks::ApiFormat, 50))
        .col(string_len(ModelStatusChecks::ApiTokenId, 36))
        .col(big_integer(ModelStatusChecks::IntervalSeconds))
        .col(boolean(ModelStatusChecks::Enabled))
        .col(timestamp_tz(ModelStatusChecks::NextDueAt))
        .col(timestamp_tz_null(ModelStatusChecks::LockedUntil))
        .col(string_len_null(ModelStatusChecks::LastStatus, 20))
        .col(timestamp_tz_null(ModelStatusChecks::LastCheckedAt))
        .col(big_integer_null(ModelStatusChecks::LastLatencyMs))
        .col(text_null(ModelStatusChecks::LastMessage))
        .col(timestamp_tz(ModelStatusChecks::CreatedAt))
        .col(timestamp_tz(ModelStatusChecks::UpdatedAt))
        .foreign_key(&mut model_fk)
        .foreign_key(&mut token_fk)
        .to_owned()
}

fn runs_table() -> TableCreateStatement {
    let mut check_fk = check_fk("fk_model_status_runs_check", ModelStatusCheckRuns::Table, ModelStatusCheckRuns::CheckId);
    Table::create()
        .table(ModelStatusCheckRuns::Table)
        .if_not_exists()
        .col(string_len(ModelStatusCheckRuns::Id, 36).primary_key())
        .col(string_len(ModelStatusCheckRuns::CheckId, 36))
        .col(string_len(ModelStatusCheckRuns::Status, 20))
        .col(big_integer_null(ModelStatusCheckRuns::LatencyMs))
        .col(integer_null(ModelStatusCheckRuns::StatusCode))
        .col(text_null(ModelStatusCheckRuns::Message))
        .col(timestamp_tz(ModelStatusCheckRuns::CheckedAt))
        .col(timestamp_tz(ModelStatusCheckRuns::CreatedAt))
        .foreign_key(&mut check_fk)
        .to_owned()
}

fn hourly_stats_table() -> TableCreateStatement {
    let mut check_fk = check_fk(
        "fk_model_status_hourly_stats_check",
        ModelStatusCheckHourlyStats::Table,
        ModelStatusCheckHourlyStats::CheckId,
    );
    Table::create()
        .table(ModelStatusCheckHourlyStats::Table)
        .if_not_exists()
        .col(string_len(ModelStatusCheckHourlyStats::Id, 36).primary_key())
        .col(string_len(ModelStatusCheckHourlyStats::CheckId, 36))
        .col(timestamp_tz(ModelStatusCheckHourlyStats::BucketStartedAt))
        .col(big_integer(ModelStatusCheckHourlyStats::TotalCount))
        .col(big_integer(ModelStatusCheckHourlyStats::AvailableCount))
        .col(big_integer(ModelStatusCheckHourlyStats::DegradedCount))
        .col(big_integer(ModelStatusCheckHourlyStats::FailedCount))
        .col(big_integer(ModelStatusCheckHourlyStats::ErrorCount))
        .col(big_integer(ModelStatusCheckHourlyStats::LatencySumMs))
        .col(timestamp_tz(ModelStatusCheckHourlyStats::CreatedAt))
        .col(timestamp_tz(ModelStatusCheckHourlyStats::UpdatedAt))
        .foreign_key(&mut check_fk)
        .to_owned()
}

fn check_fk<T>(name: &str, table: T, col: T) -> ForeignKeyCreateStatement
where
    T: IntoIden + Copy,
{
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name(name)
        .from(table, col)
        .to(ModelStatusChecks::Table, ModelStatusChecks::Id)
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
