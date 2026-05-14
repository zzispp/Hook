use sea_orm_migration::{prelude::*, schema::*};

use super::iden::{RequestCandidates, RequestRecords};

pub(super) fn request_records_table() -> TableCreateStatement {
    Table::create()
        .table(RequestRecords::Table)
        .if_not_exists()
        .col(string_len(RequestRecords::RequestId, 64).primary_key())
        .col(string_len_null(RequestRecords::TokenId, 36))
        .col(string_len_null(RequestRecords::UserIdSnapshot, 36))
        .col(string_len_null(RequestRecords::UsernameSnapshot, 100))
        .col(string_len_null(RequestRecords::TokenNameSnapshot, 100))
        .col(string_len_null(RequestRecords::TokenPrefixSnapshot, 32))
        .col(string_len_null(RequestRecords::GroupCode, 64))
        .col(string_len_null(RequestRecords::GlobalModelId, 36))
        .col(string_len_null(RequestRecords::ModelNameSnapshot, 100))
        .col(string_len_null(RequestRecords::ProviderId, 36))
        .col(string_len_null(RequestRecords::ProviderNameSnapshot, 100))
        .col(string_len_null(RequestRecords::EndpointId, 36))
        .col(string_len_null(RequestRecords::KeyId, 36))
        .col(string_len_null(RequestRecords::ProviderKeyNameSnapshot, 100))
        .col(string_len_null(RequestRecords::ProviderKeyPreviewSnapshot, 32))
        .col(string_len(RequestRecords::ClientApiFormat, 50))
        .col(string_len_null(RequestRecords::ProviderApiFormat, 50))
        .col(string_len(RequestRecords::RequestType, 40))
        .col(boolean(RequestRecords::IsStream))
        .col(boolean(RequestRecords::HasFailover))
        .col(boolean(RequestRecords::HasRetry))
        .col(string_len(RequestRecords::Status, 40))
        .col(string_len(RequestRecords::BillingStatus, 40))
        .col(integer_null(RequestRecords::ClientStatusCode))
        .col(string_len_null(RequestRecords::ClientErrorType, 100))
        .col(text_null(RequestRecords::ClientErrorMessage))
        .col(string_len_null(RequestRecords::TerminationOrigin, 60))
        .col(text_null(RequestRecords::TerminationReason))
        .col(string_len_null(RequestRecords::StreamEndReason, 60))
        .col(big_integer_null(RequestRecords::PromptTokens))
        .col(big_integer_null(RequestRecords::CompletionTokens))
        .col(big_integer_null(RequestRecords::TotalTokens))
        .col(big_integer_null(RequestRecords::CacheCreationInputTokens))
        .col(big_integer_null(RequestRecords::CacheReadInputTokens))
        .col(string_len_null(RequestRecords::CostCurrency, 3))
        .col(decimal_len_null(RequestRecords::TokenCost, 20, 8))
        .col(decimal_len_null(RequestRecords::BaseCost, 20, 8))
        .col(decimal_len_null(RequestRecords::TotalCost, 20, 8))
        .col(decimal_len_null(RequestRecords::BillingMultiplier, 20, 8))
        .col(big_integer_null(RequestRecords::FirstByteTimeMs))
        .col(big_integer_null(RequestRecords::TotalLatencyMs))
        .col(big_integer(RequestRecords::CandidateCount))
        .col(text_null(RequestRecords::RequestHeaders))
        .col(text_null(RequestRecords::RequestBody))
        .col(text_null(RequestRecords::ClientResponseHeaders))
        .col(text_null(RequestRecords::ClientResponseBody))
        .col(timestamp_tz(RequestRecords::CreatedAt))
        .col(timestamp_tz_null(RequestRecords::StartedAt))
        .col(timestamp_tz_null(RequestRecords::FinishedAt))
        .col(timestamp_tz(RequestRecords::UpdatedAt))
        .to_owned()
}

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
        .col(string_len_null(RequestCandidates::ProviderNameSnapshot, 100))
        .col(string_len_null(RequestCandidates::EndpointId, 36))
        .col(string_len_null(RequestCandidates::EndpointNameSnapshot, 50))
        .col(string_len_null(RequestCandidates::KeyId, 36))
        .col(string_len_null(RequestCandidates::KeyNameSnapshot, 100))
        .col(string_len_null(RequestCandidates::KeyPreviewSnapshot, 32))
        .col(string_len(RequestCandidates::ClientApiFormat, 50))
        .col(string_len_null(RequestCandidates::ProviderApiFormat, 50))
        .col(boolean(RequestCandidates::NeedsConversion))
        .col(boolean(RequestCandidates::IsStream))
        .col(text_null(RequestCandidates::ProviderRequestHeaders))
        .col(text_null(RequestCandidates::ProviderRequestBody))
        .col(text_null(RequestCandidates::ProviderResponseHeaders))
        .col(text_null(RequestCandidates::ProviderResponseBody))
        .col(integer(RequestCandidates::CandidateIndex))
        .col(integer(RequestCandidates::RetryIndex))
        .col(string_len(RequestCandidates::Status, 40))
        .col(text_null(RequestCandidates::SkipReason))
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
        .col(string_len_null(RequestCandidates::ErrorCode, 120))
        .col(string_len_null(RequestCandidates::ErrorParam, 160))
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
