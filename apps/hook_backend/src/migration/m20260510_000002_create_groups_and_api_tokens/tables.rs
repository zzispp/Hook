use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

pub(super) fn group_token_tables() -> Vec<TableCreateStatement> {
    vec![billing_groups_table(), api_tokens_table()]
}

pub(super) fn group_token_indices() -> Vec<IndexCreateStatement> {
    vec![
        index("index_billing_groups_by_code", BillingGroups::Table, BillingGroups::Code, true),
        index("index_billing_groups_by_active", BillingGroups::Table, BillingGroups::IsActive, false),
        index("index_api_tokens_by_hash", ApiTokens::Table, ApiTokens::TokenHash, true),
        index("index_api_tokens_by_user_id", ApiTokens::Table, ApiTokens::UserId, false),
        index("index_api_tokens_by_token_type", ApiTokens::Table, ApiTokens::TokenType, false),
        index("index_api_tokens_by_group_code", ApiTokens::Table, ApiTokens::GroupCode, false),
    ]
}

fn billing_groups_table() -> TableCreateStatement {
    Table::create()
        .table(BillingGroups::Table)
        .if_not_exists()
        .col(string_len(BillingGroups::Id, 36).primary_key())
        .col(string_len(BillingGroups::Code, 64))
        .col(string_len(BillingGroups::Name, 100))
        .col(text_null(BillingGroups::Description))
        .col(decimal_len(BillingGroups::BillingMultiplier, 20, 8))
        .col(boolean(BillingGroups::IsActive))
        .col(boolean(BillingGroups::IsSystem))
        .col(big_integer(BillingGroups::SortOrder))
        .col(timestamp_tz(BillingGroups::CreatedAt))
        .col(timestamp_tz(BillingGroups::UpdatedAt))
        .to_owned()
}

fn api_tokens_table() -> TableCreateStatement {
    Table::create()
        .table(ApiTokens::Table)
        .if_not_exists()
        .col(string_len(ApiTokens::Id, 36).primary_key())
        .col(string_len(ApiTokens::UserId, 36))
        .col(string_len(ApiTokens::TokenType, 20).default("user"))
        .col(string_len(ApiTokens::Name, 100))
        .col(string_len(ApiTokens::TokenValue, 255))
        .col(string_len(ApiTokens::TokenHash, 64))
        .col(string_len(ApiTokens::TokenPrefix, 32))
        .col(string_len(ApiTokens::GroupCode, 64))
        .col(timestamp_tz_null(ApiTokens::ExpiresAt))
        .col(string_len(ApiTokens::ModelAccessMode, 20))
        .col(text(ApiTokens::AllowedModelIds))
        .col(big_integer_null(ApiTokens::RateLimitRpm))
        .col(decimal_len_null(ApiTokens::QuotaLimit, 20, 8))
        .col(decimal_len(ApiTokens::UsedQuota, 20, 8))
        .col(big_integer(ApiTokens::RequestCount).default(0))
        .col(boolean(ApiTokens::IsActive))
        .col(timestamp_tz_null(ApiTokens::LastUsedAt))
        .col(timestamp_tz(ApiTokens::CreatedAt))
        .col(timestamp_tz(ApiTokens::UpdatedAt))
        .to_owned()
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

fn index<T, C>(name: &str, table: T, column: C, unique: bool) -> IndexCreateStatement
where
    T: Iden + 'static,
    C: Iden + 'static,
{
    let mut index = Index::create();
    index.name(name).table(table).col(column).if_not_exists();
    if unique {
        index.unique();
    }
    index.to_owned()
}
