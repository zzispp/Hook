use sea_orm_migration::{prelude::*, schema::*};

use super::{
    iden::*, model_status_tables, performance_monitoring_tables, provider_key_group_tables, request_candidate_tables, scheduler_tables, setting_tables,
    translation_tables,
};

pub(super) fn domain_tables() -> Vec<TableCreateStatement> {
    let mut tables = vec![
        billing_groups_table(),
        providers_table(),
        provider_endpoints_table(),
        provider_api_keys_table(),
        provider_models_table(),
        provider_model_costs_table(),
        provider_quick_import_sources_table(),
        provider_quick_import_keys_table(),
        provider_quick_import_key_models_table(),
        provider_quick_import_sync_events_table(),
        billing_rules_table(),
        dimension_collectors_table(),
        provider_cooldowns_table(),
        provider_cooldown_events_table(),
        billing_group_providers_table(),
        billing_group_provider_keys_table(),
        billing_group_user_groups_table(),
        setting_tables::system_settings_table(),
        translation_tables::translation_languages_table(),
        translation_tables::translation_entries_table(),
        api_tokens_table(),
        billing_group_models_table(),
        request_candidate_tables::request_records_table(),
        request_candidate_tables::dashboard_cost_analysis_buckets_table(),
        request_candidate_tables::request_candidates_table(),
        usage_flush_batches_table(),
    ];
    tables.extend(provider_key_group_tables::provider_key_group_tables());
    tables.extend(scheduler_tables::scheduler_tables());
    tables.extend(performance_monitoring_tables::performance_monitoring_tables());
    tables.extend(model_status_tables::model_status_tables());
    tables
}

fn billing_group_user_groups_table() -> TableCreateStatement {
    let mut billing_group_fk = billing_group_user_group_billing_group_fk();
    let mut user_group_fk = billing_group_user_group_user_group_fk();
    Table::create()
        .table(BillingGroupUserGroups::Table)
        .if_not_exists()
        .col(string_len(BillingGroupUserGroups::Id, 36).primary_key())
        .col(string_len(BillingGroupUserGroups::BillingGroupCode, 64))
        .col(string_len(BillingGroupUserGroups::UserGroupCode, 64))
        .col(timestamp_tz(BillingGroupUserGroups::CreatedAt))
        .col(timestamp_tz(BillingGroupUserGroups::UpdatedAt))
        .foreign_key(&mut billing_group_fk)
        .foreign_key(&mut user_group_fk)
        .to_owned()
}

fn billing_group_user_group_billing_group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_user_groups_billing_group")
        .from(BillingGroupUserGroups::Table, BillingGroupUserGroups::BillingGroupCode)
        .to(BillingGroups::Table, BillingGroups::Code)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn billing_group_user_group_user_group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_user_groups_user_group")
        .from(BillingGroupUserGroups::Table, BillingGroupUserGroups::UserGroupCode)
        .to(UserGroups::Table, UserGroups::Code);
    foreign_key
}

fn billing_rules_table() -> TableCreateStatement {
    let mut global_model_fk = billing_rule_global_model_fk();
    let mut model_fk = billing_rule_model_fk();
    Table::create()
        .table(BillingRules::Table)
        .if_not_exists()
        .col(string_len(BillingRules::Id, 36).primary_key())
        .col(string_len_null(BillingRules::GlobalModelId, 36))
        .col(string_len_null(BillingRules::ModelId, 36))
        .col(string_len(BillingRules::Name, 100))
        .col(string_len(BillingRules::TaskType, 20).default("chat"))
        .col(text(BillingRules::Expression))
        .col(text(BillingRules::Variables).default("{}"))
        .col(text(BillingRules::DimensionMappings).default("{}"))
        .col(boolean(BillingRules::IsEnabled).default(true))
        .col(timestamp_tz(BillingRules::CreatedAt))
        .col(timestamp_tz(BillingRules::UpdatedAt))
        .check(Expr::cust("(global_model_id IS NOT NULL AND model_id IS NULL) OR (global_model_id IS NULL AND model_id IS NOT NULL)").into_condition())
        .foreign_key(&mut global_model_fk)
        .foreign_key(&mut model_fk)
        .to_owned()
}

fn dimension_collectors_table() -> TableCreateStatement {
    Table::create()
        .table(DimensionCollectors::Table)
        .if_not_exists()
        .col(string_len(DimensionCollectors::Id, 36).primary_key())
        .col(string_len(DimensionCollectors::ApiFormat, 50))
        .col(string_len(DimensionCollectors::TaskType, 20))
        .col(string_len(DimensionCollectors::DimensionName, 100))
        .col(string_len(DimensionCollectors::SourceType, 20))
        .col(string_len_null(DimensionCollectors::SourcePath, 200))
        .col(string_len(DimensionCollectors::ValueType, 20).default("float"))
        .col(text_null(DimensionCollectors::TransformExpression))
        .col(string_len_null(DimensionCollectors::DefaultValue, 100))
        .col(integer(DimensionCollectors::Priority).default(0))
        .col(boolean(DimensionCollectors::IsEnabled).default(true))
        .col(timestamp_tz(DimensionCollectors::CreatedAt))
        .col(timestamp_tz(DimensionCollectors::UpdatedAt))
        .check(
            Expr::cust(
                "(source_type = 'computed' AND source_path IS NULL AND transform_expression IS NOT NULL) OR (source_type != 'computed' AND source_path IS NOT NULL)",
            )
            .into_condition(),
        )
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
        .col(string_len(Providers::ProviderOrigin, 32).default("manual"))
        .col(integer_null(Providers::MaxRetries))
        .col(double_null(Providers::RequestTimeoutSeconds))
        .col(double_null(Providers::StreamFirstByteTimeoutSeconds))
        .col(double_null(Providers::StreamIdleTimeoutSeconds))
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
        .col(text(ProviderApiKeys::ApiFormats))
        .col(text(ProviderApiKeys::AllowedModelIds))
        .col(text_null(ProviderApiKeys::Capabilities))
        .col(text(ProviderApiKeys::EncryptedApiKey))
        .col(text_null(ProviderApiKeys::Note))
        .col(integer(ProviderApiKeys::InternalPriority))
        .col(text(ProviderApiKeys::GlobalPriorityByFormat))
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
        .col(boolean(ProviderModels::IsActive))
        .col(text_null(ProviderModels::Config))
        .col(timestamp_tz(ProviderModels::CreatedAt))
        .col(timestamp_tz(ProviderModels::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .foreign_key(&mut global_model_fk)
        .to_owned()
}

fn provider_model_costs_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk("fk_provider_model_costs_provider", ProviderModelCosts::Table, ProviderModelCosts::ProviderId);
    let mut key_fk = provider_model_cost_key_fk();
    let mut model_fk = provider_model_cost_model_fk();
    Table::create()
        .table(ProviderModelCosts::Table)
        .if_not_exists()
        .col(string_len(ProviderModelCosts::Id, 36).primary_key())
        .col(string_len(ProviderModelCosts::ProviderId, 36))
        .col(string_len(ProviderModelCosts::KeyId, 36))
        .col(string_len(ProviderModelCosts::ProviderModelId, 36))
        .col(string_len(ProviderModelCosts::CostMode, 20))
        .col(decimal_len_null(ProviderModelCosts::PricePerRequest, 20, 8))
        .col(decimal_len_null(ProviderModelCosts::InputPricePerMillion, 20, 8))
        .col(decimal_len_null(ProviderModelCosts::OutputPricePerMillion, 20, 8))
        .col(decimal_len_null(ProviderModelCosts::CacheCreationPricePerMillion, 20, 8))
        .col(decimal_len_null(ProviderModelCosts::CacheReadPricePerMillion, 20, 8))
        .col(timestamp_tz(ProviderModelCosts::CreatedAt))
        .col(timestamp_tz(ProviderModelCosts::UpdatedAt))
        .check(provider_model_cost_mode_check())
        .foreign_key(&mut provider_fk)
        .foreign_key(&mut key_fk)
        .foreign_key(&mut model_fk)
        .to_owned()
}

fn provider_quick_import_sources_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk(
        "fk_provider_quick_import_sources_provider",
        ProviderQuickImportSources::Table,
        ProviderQuickImportSources::ProviderId,
    );
    Table::create()
        .table(ProviderQuickImportSources::Table)
        .if_not_exists()
        .col(string_len(ProviderQuickImportSources::Id, 36).primary_key())
        .col(string_len(ProviderQuickImportSources::ProviderId, 36))
        .col(string_len(ProviderQuickImportSources::SourceKind, 32))
        .col(text(ProviderQuickImportSources::BaseUrl))
        .col(text(ProviderQuickImportSources::EncryptedSystemAccessToken))
        .col(string_len(ProviderQuickImportSources::UserId, 100))
        .col(decimal_len(ProviderQuickImportSources::RechargeMultiplier, 20, 8))
        .col(boolean(ProviderQuickImportSources::AutoSyncEnabled))
        .col(string_len(ProviderQuickImportSources::CostSyncMode, 32))
        .col(string_len(ProviderQuickImportSources::UpstreamAnomalyAction, 32))
        .col(string_len(ProviderQuickImportSources::TokenDeletedAction, 32))
        .col(string_len(ProviderQuickImportSources::TokenDisabledAction, 32))
        .col(string_len(ProviderQuickImportSources::GroupRemovedAction, 32))
        .col(string_len(ProviderQuickImportSources::GroupChangedAction, 32))
        .col(string_len(ProviderQuickImportSources::KeyUnavailableAction, 32))
        .col(string_len(ProviderQuickImportSources::ModelRemovedAction, 32))
        .col(string_len(ProviderQuickImportSources::FetchFailureAction, 32))
        .col(integer(ProviderQuickImportSources::FetchFailureDisableThreshold))
        .col(string_len_null(ProviderQuickImportSources::LastStatus, 64))
        .col(text_null(ProviderQuickImportSources::LastError))
        .col(timestamp_tz_null(ProviderQuickImportSources::LastSyncedAt))
        .col(integer(ProviderQuickImportSources::ConsecutiveFailures).default(0))
        .col(timestamp_tz(ProviderQuickImportSources::CreatedAt))
        .col(timestamp_tz(ProviderQuickImportSources::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .to_owned()
}

fn provider_quick_import_keys_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk(
        "fk_provider_quick_import_keys_provider",
        ProviderQuickImportKeys::Table,
        ProviderQuickImportKeys::ProviderId,
    );
    let mut source_fk = quick_import_key_source_fk();
    let mut key_fk = quick_import_key_provider_key_fk();
    Table::create()
        .table(ProviderQuickImportKeys::Table)
        .if_not_exists()
        .col(string_len(ProviderQuickImportKeys::Id, 36).primary_key())
        .col(string_len(ProviderQuickImportKeys::ProviderId, 36))
        .col(string_len(ProviderQuickImportKeys::SourceId, 36))
        .col(string_len(ProviderQuickImportKeys::KeyId, 36))
        .col(string_len(ProviderQuickImportKeys::UpstreamTokenId, 100))
        .col(string_len(ProviderQuickImportKeys::UpstreamTokenName, 100))
        .col(string_len(ProviderQuickImportKeys::UpstreamMaskedKey, 200))
        .col(string_len_null(ProviderQuickImportKeys::UpstreamGroup, 100))
        .col(decimal_len(ProviderQuickImportKeys::UpstreamGroupRatio, 20, 8))
        .col(decimal_len(ProviderQuickImportKeys::EffectiveCostMultiplier, 20, 8))
        .col(text(ProviderQuickImportKeys::SyncStatuses))
        .col(text_null(ProviderQuickImportKeys::LastSyncError))
        .col(timestamp_tz_null(ProviderQuickImportKeys::LastSyncedAt))
        .col(timestamp_tz(ProviderQuickImportKeys::CreatedAt))
        .col(timestamp_tz(ProviderQuickImportKeys::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .foreign_key(&mut source_fk)
        .foreign_key(&mut key_fk)
        .to_owned()
}

fn provider_quick_import_key_models_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk(
        "fk_provider_quick_import_key_models_provider",
        ProviderQuickImportKeyModels::Table,
        ProviderQuickImportKeyModels::ProviderId,
    );
    let mut source_fk = quick_import_key_model_source_fk();
    let mut key_fk = quick_import_key_model_key_fk();
    let mut global_model_fk = quick_import_key_model_global_model_fk();
    Table::create()
        .table(ProviderQuickImportKeyModels::Table)
        .if_not_exists()
        .col(string_len(ProviderQuickImportKeyModels::Id, 36).primary_key())
        .col(string_len(ProviderQuickImportKeyModels::ProviderId, 36))
        .col(string_len(ProviderQuickImportKeyModels::SourceId, 36))
        .col(string_len(ProviderQuickImportKeyModels::KeyId, 36))
        .col(string_len(ProviderQuickImportKeyModels::UpstreamModelId, 200))
        .col(string_len(ProviderQuickImportKeyModels::GlobalModelId, 36))
        .col(timestamp_tz(ProviderQuickImportKeyModels::CreatedAt))
        .col(timestamp_tz(ProviderQuickImportKeyModels::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .foreign_key(&mut source_fk)
        .foreign_key(&mut key_fk)
        .foreign_key(&mut global_model_fk)
        .to_owned()
}

fn provider_quick_import_sync_events_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk(
        "fk_provider_quick_import_sync_events_provider",
        ProviderQuickImportSyncEvents::Table,
        ProviderQuickImportSyncEvents::ProviderId,
    );
    let mut source_fk = quick_import_sync_event_source_fk();
    let mut key_fk = quick_import_sync_event_key_fk();
    Table::create()
        .table(ProviderQuickImportSyncEvents::Table)
        .if_not_exists()
        .col(string_len(ProviderQuickImportSyncEvents::Id, 36).primary_key())
        .col(string_len(ProviderQuickImportSyncEvents::ProviderId, 36))
        .col(string_len(ProviderQuickImportSyncEvents::SourceId, 36))
        .col(string_len_null(ProviderQuickImportSyncEvents::KeyId, 36))
        .col(string_len(ProviderQuickImportSyncEvents::Status, 64))
        .col(text(ProviderQuickImportSyncEvents::Title))
        .col(text(ProviderQuickImportSyncEvents::Detail))
        .col(timestamp_tz(ProviderQuickImportSyncEvents::CreatedAt))
        .foreign_key(&mut provider_fk)
        .foreign_key(&mut source_fk)
        .foreign_key(&mut key_fk)
        .to_owned()
}

fn provider_cooldowns_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk("fk_provider_cooldowns_provider", ProviderCooldowns::Table, ProviderCooldowns::ProviderId);
    Table::create()
        .table(ProviderCooldowns::Table)
        .if_not_exists()
        .col(string_len(ProviderCooldowns::ProviderId, 36).primary_key())
        .col(string_len(ProviderCooldowns::ProviderNameSnapshot, 100))
        .col(integer(ProviderCooldowns::StatusCode))
        .col(big_integer(ProviderCooldowns::ObservedCount))
        .col(big_integer(ProviderCooldowns::ThresholdCount))
        .col(big_integer(ProviderCooldowns::WindowSeconds))
        .col(big_integer(ProviderCooldowns::CooldownSeconds))
        .col(timestamp_tz(ProviderCooldowns::TriggeredAt))
        .col(timestamp_tz(ProviderCooldowns::CooldownUntil))
        .col(timestamp_tz_null(ProviderCooldowns::ReleasedAt))
        .col(string_len(ProviderCooldowns::RequestId, 64))
        .col(integer(ProviderCooldowns::CandidateIndex))
        .col(integer(ProviderCooldowns::RetryIndex))
        .col(string_len_null(ProviderCooldowns::EndpointId, 36))
        .col(string_len_null(ProviderCooldowns::EndpointNameSnapshot, 50))
        .col(string_len_null(ProviderCooldowns::KeyId, 36))
        .col(string_len_null(ProviderCooldowns::KeyNameSnapshot, 100))
        .col(string_len_null(ProviderCooldowns::ErrorType, 100))
        .col(text_null(ProviderCooldowns::ErrorMessage))
        .col(text_null(ProviderCooldowns::ErrorCode))
        .col(string_len_null(ProviderCooldowns::ErrorParam, 160))
        .col(timestamp_tz(ProviderCooldowns::CreatedAt))
        .col(timestamp_tz(ProviderCooldowns::UpdatedAt))
        .foreign_key(&mut provider_fk)
        .to_owned()
}

fn provider_cooldown_events_table() -> TableCreateStatement {
    let mut provider_fk = provider_fk(
        "fk_provider_cooldown_events_provider",
        ProviderCooldownEvents::Table,
        ProviderCooldownEvents::ProviderId,
    );
    Table::create()
        .table(ProviderCooldownEvents::Table)
        .if_not_exists()
        .col(string_len(ProviderCooldownEvents::Id, 36).primary_key())
        .col(string_len(ProviderCooldownEvents::ProviderId, 36))
        .col(string_len(ProviderCooldownEvents::ProviderNameSnapshot, 100))
        .col(integer(ProviderCooldownEvents::StatusCode))
        .col(big_integer(ProviderCooldownEvents::ObservedCount))
        .col(big_integer(ProviderCooldownEvents::ThresholdCount))
        .col(big_integer(ProviderCooldownEvents::WindowSeconds))
        .col(big_integer(ProviderCooldownEvents::CooldownSeconds))
        .col(timestamp_tz(ProviderCooldownEvents::TriggeredAt))
        .col(timestamp_tz(ProviderCooldownEvents::CooldownUntil))
        .col(string_len(ProviderCooldownEvents::RequestId, 64))
        .col(integer(ProviderCooldownEvents::CandidateIndex))
        .col(integer(ProviderCooldownEvents::RetryIndex))
        .col(string_len_null(ProviderCooldownEvents::EndpointId, 36))
        .col(string_len_null(ProviderCooldownEvents::EndpointNameSnapshot, 50))
        .col(string_len_null(ProviderCooldownEvents::KeyId, 36))
        .col(string_len_null(ProviderCooldownEvents::KeyNameSnapshot, 100))
        .col(string_len_null(ProviderCooldownEvents::ErrorType, 100))
        .col(text_null(ProviderCooldownEvents::ErrorMessage))
        .col(text_null(ProviderCooldownEvents::ErrorCode))
        .col(string_len_null(ProviderCooldownEvents::ErrorParam, 160))
        .col(timestamp_tz(ProviderCooldownEvents::CreatedAt))
        .foreign_key(&mut provider_fk)
        .to_owned()
}

fn billing_group_providers_table() -> TableCreateStatement {
    let mut group_fk = billing_group_provider_billing_group_fk();
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

fn billing_group_provider_keys_table() -> TableCreateStatement {
    let mut group_fk = billing_group_provider_key_group_fk();
    let mut key_fk = billing_group_provider_key_key_fk();
    Table::create()
        .table(BillingGroupProviderKeys::Table)
        .if_not_exists()
        .col(string_len(BillingGroupProviderKeys::Id, 36).primary_key())
        .col(string_len(BillingGroupProviderKeys::GroupCode, 64))
        .col(string_len(BillingGroupProviderKeys::ProviderKeyId, 36))
        .col(timestamp_tz(BillingGroupProviderKeys::CreatedAt))
        .col(timestamp_tz(BillingGroupProviderKeys::UpdatedAt))
        .foreign_key(&mut group_fk)
        .foreign_key(&mut key_fk)
        .to_owned()
}

fn api_tokens_table() -> TableCreateStatement {
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
        .to_owned()
}

fn usage_flush_batches_table() -> TableCreateStatement {
    Table::create()
        .table(UsageFlushBatches::Table)
        .if_not_exists()
        .col(string_len(UsageFlushBatches::Id, 36).primary_key())
        .col(string_len(UsageFlushBatches::UsageKind, 20))
        .col(big_integer(UsageFlushBatches::RecordCount))
        .col(timestamp_tz(UsageFlushBatches::CreatedAt))
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

fn provider_model_cost_key_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_model_costs_key")
        .from(ProviderModelCosts::Table, ProviderModelCosts::KeyId)
        .to(ProviderApiKeys::Table, ProviderApiKeys::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn provider_model_cost_model_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_model_costs_model")
        .from(ProviderModelCosts::Table, ProviderModelCosts::ProviderModelId)
        .to(ProviderModels::Table, ProviderModels::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn quick_import_key_source_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_quick_import_keys_source")
        .from(ProviderQuickImportKeys::Table, ProviderQuickImportKeys::SourceId)
        .to(ProviderQuickImportSources::Table, ProviderQuickImportSources::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn quick_import_key_provider_key_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_quick_import_keys_key")
        .from(ProviderQuickImportKeys::Table, ProviderQuickImportKeys::KeyId)
        .to(ProviderApiKeys::Table, ProviderApiKeys::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn quick_import_key_model_source_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_quick_import_key_models_source")
        .from(ProviderQuickImportKeyModels::Table, ProviderQuickImportKeyModels::SourceId)
        .to(ProviderQuickImportSources::Table, ProviderQuickImportSources::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn quick_import_key_model_key_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_quick_import_key_models_key")
        .from(ProviderQuickImportKeyModels::Table, ProviderQuickImportKeyModels::KeyId)
        .to(ProviderApiKeys::Table, ProviderApiKeys::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn quick_import_key_model_global_model_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_quick_import_key_models_global_model")
        .from(ProviderQuickImportKeyModels::Table, ProviderQuickImportKeyModels::GlobalModelId)
        .to(GlobalModels::Table, GlobalModels::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn quick_import_sync_event_source_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_quick_import_sync_events_source")
        .from(ProviderQuickImportSyncEvents::Table, ProviderQuickImportSyncEvents::SourceId)
        .to(ProviderQuickImportSources::Table, ProviderQuickImportSources::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn quick_import_sync_event_key_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_provider_quick_import_sync_events_key")
        .from(ProviderQuickImportSyncEvents::Table, ProviderQuickImportSyncEvents::KeyId)
        .to(ProviderApiKeys::Table, ProviderApiKeys::Id)
        .on_delete(ForeignKeyAction::SetNull);
    foreign_key
}

fn provider_model_cost_mode_check() -> SimpleExpr {
    Expr::cust(
        "(cost_mode = 'per_request' AND price_per_request IS NOT NULL \
         AND input_price_per_million IS NULL AND output_price_per_million IS NULL \
         AND cache_creation_price_per_million IS NULL AND cache_read_price_per_million IS NULL) \
         OR (cost_mode = 'per_token' AND price_per_request IS NULL \
         AND input_price_per_million IS NOT NULL AND output_price_per_million IS NOT NULL \
         AND cache_creation_price_per_million IS NOT NULL AND cache_read_price_per_million IS NOT NULL)",
    )
}

fn billing_rule_global_model_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_rules_global_model")
        .from(BillingRules::Table, BillingRules::GlobalModelId)
        .to(GlobalModels::Table, GlobalModels::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn billing_rule_model_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_rules_model")
        .from(BillingRules::Table, BillingRules::ModelId)
        .to(ProviderModels::Table, ProviderModels::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn billing_group_provider_billing_group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_providers_group")
        .from(BillingGroupProviders::Table, BillingGroupProviders::GroupCode)
        .to(BillingGroups::Table, BillingGroups::Code)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn billing_group_provider_key_group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_provider_keys_group")
        .from(BillingGroupProviderKeys::Table, BillingGroupProviderKeys::GroupCode)
        .to(BillingGroups::Table, BillingGroups::Code)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn billing_group_provider_key_key_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_provider_keys_key")
        .from(BillingGroupProviderKeys::Table, BillingGroupProviderKeys::ProviderKeyId)
        .to(ProviderApiKeys::Table, ProviderApiKeys::Id)
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
