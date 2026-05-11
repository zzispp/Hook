use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

pub(super) fn domain_tables() -> Vec<TableCreateStatement> {
    vec![
        billing_groups_table(),
        providers_table(),
        provider_endpoints_table(),
        provider_api_keys_table(),
        provider_models_table(),
        billing_group_providers_table(),
        system_settings_table(),
        translation_languages_table(),
        translation_entries_table(),
        api_tokens_table(),
        billing_group_models_table(),
        request_candidates_table(),
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

fn providers_table() -> TableCreateStatement {
    Table::create()
        .table(Providers::Table)
        .if_not_exists()
        .col(string_len(Providers::Id, 36).primary_key())
        .col(string_len(Providers::Name, 100))
        .col(string_len(Providers::ProviderType, 50))
        .col(integer_null(Providers::MaxRetries))
        .col(double_null(Providers::RequestTimeoutSeconds))
        .col(double_null(Providers::StreamFirstByteTimeoutSeconds))
        .col(integer(Providers::Priority))
        .col(boolean(Providers::KeepPriorityOnConversion))
        .col(boolean(Providers::EnableFormatConversion))
        .col(boolean(Providers::IsActive))
        .col(timestamp_tz(Providers::CreatedAt))
        .col(timestamp_tz(Providers::UpdatedAt))
        .to_owned()
}

fn provider_endpoints_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk("fk_provider_endpoints_provider", ProviderEndpoints::Table, ProviderEndpoints::ProviderId);
    Table::create()
        .table(ProviderEndpoints::Table)
        .if_not_exists()
        .col(string_len(ProviderEndpoints::Id, 36).primary_key())
        .col(string_len(ProviderEndpoints::ProviderId, 36))
        .col(string_len(ProviderEndpoints::ApiFormat, 50))
        .col(string_len(ProviderEndpoints::BaseUrl, 500))
        .col(string_len_null(ProviderEndpoints::CustomPath, 200))
        .col(integer_null(ProviderEndpoints::MaxRetries))
        .col(boolean(ProviderEndpoints::IsActive))
        .col(text_null(ProviderEndpoints::FormatAcceptanceConfig))
        .col(text_null(ProviderEndpoints::HeaderRules))
        .col(text_null(ProviderEndpoints::BodyRules))
        .col(timestamp_tz(ProviderEndpoints::CreatedAt))
        .col(timestamp_tz(ProviderEndpoints::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .to_owned()
}

fn provider_api_keys_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk("fk_provider_api_keys_provider", ProviderApiKeys::Table, ProviderApiKeys::ProviderId);
    Table::create()
        .table(ProviderApiKeys::Table)
        .if_not_exists()
        .col(string_len(ProviderApiKeys::Id, 36).primary_key())
        .col(string_len(ProviderApiKeys::ProviderId, 36))
        .col(string_len(ProviderApiKeys::Name, 100))
        .col(text(ProviderApiKeys::EncryptedApiKey))
        .col(text_null(ProviderApiKeys::Note))
        .col(text_null(ProviderApiKeys::ApiFormats))
        .col(integer(ProviderApiKeys::InternalPriority))
        .col(integer_null(ProviderApiKeys::RpmLimit))
        .col(integer_null(ProviderApiKeys::LearnedRpmLimit))
        .col(integer(ProviderApiKeys::CacheTtlMinutes))
        .col(integer(ProviderApiKeys::MaxProbeIntervalMinutes))
        .col(boolean(ProviderApiKeys::TimeRangeEnabled))
        .col(string_len_null(ProviderApiKeys::TimeRangeStart, 16))
        .col(string_len_null(ProviderApiKeys::TimeRangeEnd, 16))
        .col(text_null(ProviderApiKeys::HealthByFormat))
        .col(text_null(ProviderApiKeys::CircuitBreakerByFormat))
        .col(boolean(ProviderApiKeys::IsActive))
        .col(timestamp_tz(ProviderApiKeys::CreatedAt))
        .col(timestamp_tz(ProviderApiKeys::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .to_owned()
}

fn provider_models_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk("fk_provider_models_provider", ProviderModels::Table, ProviderModels::ProviderId);
    let mut global_model_fk = provider_model_global_model_fk();
    Table::create()
        .table(ProviderModels::Table)
        .if_not_exists()
        .col(string_len(ProviderModels::Id, 36).primary_key())
        .col(string_len(ProviderModels::ProviderId, 36))
        .col(string_len(ProviderModels::GlobalModelId, 36))
        .col(string_len(ProviderModels::ProviderModelName, 200))
        .col(text_null(ProviderModels::ProviderModelMappings))
        .col(decimal_len_null(ProviderModels::PricePerRequest, 20, 8))
        .col(text_null(ProviderModels::TieredPricing))
        .col(text_null(ProviderModels::Config))
        .col(timestamp_tz(ProviderModels::CreatedAt))
        .col(timestamp_tz(ProviderModels::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .foreign_key(&mut global_model_fk)
        .to_owned()
}

fn billing_group_providers_table() -> TableCreateStatement {
    let mut group_fk = billing_group_provider_group_fk();
    let mut provider_fk = provider_fk(
        "fk_billing_group_providers_provider",
        BillingGroupProviders::Table,
        BillingGroupProviders::ProviderId,
    );
    Table::create()
        .table(BillingGroupProviders::Table)
        .if_not_exists()
        .col(string_len(BillingGroupProviders::Id, 36).primary_key())
        .col(string_len(BillingGroupProviders::GroupCode, 64))
        .col(string_len(BillingGroupProviders::ProviderId, 36))
        .col(timestamp_tz(BillingGroupProviders::CreatedAt))
        .col(timestamp_tz(BillingGroupProviders::UpdatedAt))
        .foreign_key(&mut group_fk)
        .foreign_key(&mut provider_fk)
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
        .col(string_len(SystemSettings::SchedulingMode, 30))
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
        .col(string_len_null(ApiTokens::UserId, 36))
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

fn request_candidates_table() -> TableCreateStatement {
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
        .col(integer(RequestCandidates::CandidateIndex))
        .col(integer(RequestCandidates::RetryIndex))
        .col(string_len(RequestCandidates::Status, 40))
        .col(integer_null(RequestCandidates::StatusCode))
        .col(big_integer_null(RequestCandidates::LatencyMs))
        .col(big_integer_null(RequestCandidates::FirstByteTimeMs))
        .col(string_len_null(RequestCandidates::ErrorType, 100))
        .col(text_null(RequestCandidates::ErrorMessage))
        .col(timestamp_tz(RequestCandidates::CreatedAt))
        .col(timestamp_tz_null(RequestCandidates::StartedAt))
        .col(timestamp_tz_null(RequestCandidates::FinishedAt))
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

fn provider_fk<T, C>(name: &str, table: T, column: C) -> ForeignKeyCreateStatement
where
    T: IntoIden,
    C: IntoIden,
{
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name(name)
        .from(table, column)
        .to(Providers::Table, Providers::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn provider_model_global_model_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_models_global_model")
        .from(ProviderModels::Table, ProviderModels::GlobalModelId)
        .to(GlobalModels::Table, GlobalModels::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn billing_group_provider_group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_providers_group")
        .from(BillingGroupProviders::Table, BillingGroupProviders::GroupCode)
        .to(BillingGroups::Table, BillingGroups::Code)
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

fn double_null<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).double().null().take()
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
