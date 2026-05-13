use sea_orm_migration::{prelude::*, schema::*};

use super::iden::RequestCandidates;

pub(super) fn request_candidates_table() -> TableCreateStatement {
    Table::create()
        .table(RequestCandidates::Table)
        .if_not_exists()
        .col(string_len(RequestCandidates::Id, 36).primary_key())
        .col(string_len(RequestCandidates::RequestId, 64))
        .col(string_len_null(RequestCandidates::TokenId, 36))
        .col(string_len_null(RequestCandidates::GroupCode, 64))
        .col(string_len_null(RequestCandidates::GlobalModelId, 36))
        .col(string_len_null(RequestCandidates::ProviderId, 36))
        .col(string_len_null(RequestCandidates::EndpointId, 36))
        .col(string_len_null(RequestCandidates::KeyId, 36))
        .col(string_len(RequestCandidates::ClientApiFormat, 50))
        .col(string_len_null(RequestCandidates::ProviderApiFormat, 50))
        .col(boolean(RequestCandidates::NeedsConversion))
        .col(boolean(RequestCandidates::IsStream))
        .col(text_null(RequestCandidates::RequestHeaders))
        .col(text_null(RequestCandidates::RequestBody))
        .col(text_null(RequestCandidates::ResponseBody))
        .col(integer(RequestCandidates::CandidateIndex))
        .col(integer(RequestCandidates::RetryIndex))
        .col(string_len(RequestCandidates::Status, 40))
        .col(integer_null(RequestCandidates::StatusCode))
        .col(big_integer_null(RequestCandidates::PromptTokens))
        .col(big_integer_null(RequestCandidates::CompletionTokens))
        .col(big_integer_null(RequestCandidates::TotalTokens))
        .col(big_integer_null(RequestCandidates::CacheCreationInputTokens))
        .col(big_integer_null(RequestCandidates::CacheReadInputTokens))
        .col(string_len_null(RequestCandidates::CostCurrency, 3))
        .col(decimal_len_null(RequestCandidates::TokenCost, 20, 8))
        .col(decimal_len_null(RequestCandidates::BaseCost, 20, 8))
        .col(decimal_len_null(RequestCandidates::TotalCost, 20, 8))
        .col(decimal_len_null(RequestCandidates::BillingMultiplier, 20, 8))
        .col(big_integer_null(RequestCandidates::LatencyMs))
        .col(big_integer_null(RequestCandidates::FirstByteTimeMs))
        .col(string_len_null(RequestCandidates::ErrorType, 100))
        .col(text_null(RequestCandidates::ErrorMessage))
        .col(timestamp_tz(RequestCandidates::CreatedAt))
        .col(timestamp_tz_null(RequestCandidates::StartedAt))
        .col(timestamp_tz_null(RequestCandidates::FinishedAt))
        .to_owned()
}

fn integer<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).integer().not_null().take()
}

fn integer_null<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).integer().null().take()
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
