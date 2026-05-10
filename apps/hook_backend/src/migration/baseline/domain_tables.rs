use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

pub(super) fn domain_tables() -> Vec<TableCreateStatement> {
    vec![
        billing_groups_table(),
        system_settings_table(),
        translation_languages_table(),
        translation_entries_table(),
        api_tokens_table(),
        billing_group_models_table(),
    ]
}

fn translation_languages_table() -> TableCreateStatement {
    Table::create()
        .table(TranslationLanguages::Table)
        .if_not_exists()
        .col(string_len(TranslationLanguages::Code, 32).primary_key())
        .col(string_len(TranslationLanguages::Name, 120))
        .col(string_len(TranslationLanguages::NativeName, 120))
        .col(boolean(TranslationLanguages::Enabled))
        .col(boolean(TranslationLanguages::System))
        .col(big_integer(TranslationLanguages::SortOrder))
        .col(timestamp_tz(TranslationLanguages::CreatedAt))
        .col(timestamp_tz(TranslationLanguages::UpdatedAt))
        .to_owned()
}

fn translation_entries_table() -> TableCreateStatement {
    let mut language_fk = translation_language_fk();
    Table::create()
        .table(TranslationEntries::Table)
        .if_not_exists()
        .col(string_len(TranslationEntries::Id, 36).primary_key())
        .col(string_len(TranslationEntries::Namespace, 64))
        .col(string_len(TranslationEntries::GroupKey, 120))
        .col(string_len(TranslationEntries::ItemKey, 120))
        .col(string_len(TranslationEntries::LangCode, 32))
        .col(text(TranslationEntries::Value))
        .col(text_null(TranslationEntries::Description))
        .col(boolean(TranslationEntries::Enabled))
        .col(timestamp_tz(TranslationEntries::CreatedAt))
        .col(timestamp_tz(TranslationEntries::UpdatedAt))
        .foreign_key(&mut language_fk)
        .to_owned()
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
        .index(Index::create().name("index_billing_groups_by_code").col(BillingGroups::Code).unique())
        .to_owned()
}

fn system_settings_table() -> TableCreateStatement {
    Table::create()
        .table(SystemSettings::Table)
        .if_not_exists()
        .col(string_len(SystemSettings::Id, 36).primary_key())
        .col(string_len(SystemSettings::SiteName, 100))
        .col(string_len(SystemSettings::SiteSubtitle, 200))
        .col(boolean(SystemSettings::AllowRegistration))
        .col(boolean(SystemSettings::AutoDeleteExpiredTokens))
        .col(decimal_len(SystemSettings::DefaultUserGrant, 20, 8))
        .col(big_integer(SystemSettings::DefaultRateLimitRpm))
        .col(timestamp_tz(SystemSettings::CreatedAt))
        .col(timestamp_tz(SystemSettings::UpdatedAt))
        .to_owned()
}

fn api_tokens_table() -> TableCreateStatement {
    let mut user_fk = user_fk();
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
        .col(decimal_len(ApiTokens::UsedQuota, 20, 8).default(0))
        .col(big_integer(ApiTokens::RequestCount).default(0))
        .col(boolean(ApiTokens::IsActive))
        .col(timestamp_tz_null(ApiTokens::LastUsedAt))
        .col(timestamp_tz(ApiTokens::CreatedAt))
        .col(timestamp_tz(ApiTokens::UpdatedAt))
        .foreign_key(&mut user_fk)
        .to_owned()
}

fn billing_group_models_table() -> TableCreateStatement {
    let mut group_fk = group_fk();
    let mut model_fk = global_model_fk();
    Table::create()
        .table(BillingGroupModels::Table)
        .if_not_exists()
        .col(string_len(BillingGroupModels::Id, 36).primary_key())
        .col(string_len(BillingGroupModels::GroupCode, 64))
        .col(string_len(BillingGroupModels::GlobalModelId, 36))
        .col(timestamp_tz(BillingGroupModels::CreatedAt))
        .col(timestamp_tz(BillingGroupModels::UpdatedAt))
        .foreign_key(&mut group_fk)
        .foreign_key(&mut model_fk)
        .to_owned()
}

fn user_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_api_tokens_user")
        .from(ApiTokens::Table, ApiTokens::UserId)
        .to(Users::Table, Users::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_models_group")
        .from(BillingGroupModels::Table, BillingGroupModels::GroupCode)
        .to(BillingGroups::Table, BillingGroups::Code)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn global_model_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_models_global_model")
        .from(BillingGroupModels::Table, BillingGroupModels::GlobalModelId)
        .to(GlobalModels::Table, GlobalModels::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn translation_language_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_translation_entries_language")
        .from(TranslationEntries::Table, TranslationEntries::LangCode)
        .to(TranslationLanguages::Table, TranslationLanguages::Code)
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
