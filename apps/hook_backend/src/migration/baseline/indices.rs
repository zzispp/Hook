use sea_orm_migration::prelude::*;

use super::iden::*;

pub(super) fn baseline_indices() -> Vec<IndexCreateStatement> {
    let mut indices = vec![
        index("index_users_by_username", Users::Table, Users::Username, true),
        index("index_users_by_email", Users::Table, Users::Email, true),
        index("index_users_by_affiliate_code", Users::Table, Users::AffiliateCode, true),
        index("index_users_by_referrer", Users::Table, Users::ReferredByUserId, false),
        user_group_memberships_unique_index(),
        index(
            "index_user_group_memberships_by_user",
            UserGroupMemberships::Table,
            UserGroupMemberships::UserId,
            false,
        ),
        index(
            "index_user_group_memberships_by_user_group",
            UserGroupMemberships::Table,
            UserGroupMemberships::UserGroupCode,
            false,
        ),
        index("index_user_identities_by_user", UserIdentities::Table, UserIdentities::UserId, false),
        compound_index(
            "index_affiliate_relation_changes_by_user_created",
            AffiliateRelationChanges::Table,
            AffiliateRelationChanges::UserId,
            AffiliateRelationChanges::CreatedAt,
        ),
        compound_index(
            "index_affiliate_relation_changes_by_operator_created",
            AffiliateRelationChanges::Table,
            AffiliateRelationChanges::OperatorUserId,
            AffiliateRelationChanges::CreatedAt,
        ),
        compound_index_unique(
            "index_user_identities_by_provider_subject",
            UserIdentities::Table,
            UserIdentities::Provider,
            UserIdentities::ProviderSubject,
        ),
        index("index_user_groups_by_active", UserGroups::Table, UserGroups::IsActive, false),
        index(
            "index_user_password_reset_tokens_by_hash",
            UserPasswordResetTokens::Table,
            UserPasswordResetTokens::TokenHash,
            true,
        ),
        index(
            "index_user_password_reset_tokens_by_user",
            UserPasswordResetTokens::Table,
            UserPasswordResetTokens::UserId,
            false,
        ),
        index("index_api_permissions_by_code", ApiPermissions::Table, ApiPermissions::Code, true),
        index("index_menu_sections_by_code", MenuSections::Table, MenuSections::Code, true),
        index("index_menu_items_by_section_id", MenuItems::Table, MenuItems::SectionId, false),
        index("index_menu_items_by_code", MenuItems::Table, MenuItems::Code, true),
        index(
            "index_menu_api_permissions_by_api_id",
            MenuApiPermissions::Table,
            MenuApiPermissions::ApiPermissionId,
            false,
        ),
        index(
            "index_role_api_permissions_by_api_id",
            RoleApiPermissions::Table,
            RoleApiPermissions::ApiPermissionId,
            false,
        ),
        index("index_wallets_by_user_id", Wallets::Table, Wallets::UserId, true),
        index("index_wallets_by_status", Wallets::Table, Wallets::Status, false),
        compound_index(
            "index_wallet_transactions_by_wallet_created",
            WalletTransactions::Table,
            WalletTransactions::WalletId,
            WalletTransactions::CreatedAt,
        ),
        compound_index(
            "index_wallet_transactions_by_link",
            WalletTransactions::Table,
            WalletTransactions::LinkType,
            WalletTransactions::LinkId,
        ),
        compound_index(
            "index_wallet_transactions_by_category_created",
            WalletTransactions::Table,
            WalletTransactions::Category,
            WalletTransactions::CreatedAt,
        ),
        index("index_card_code_types_by_name", CardCodeTypes::Table, CardCodeTypes::Name, true),
        index("index_card_code_types_by_status", CardCodeTypes::Table, CardCodeTypes::Status, false),
        index("index_card_codes_by_code", CardCodes::Table, CardCodes::Code, true),
        compound_index("index_card_codes_by_status_created", CardCodes::Table, CardCodes::Status, CardCodes::CreatedAt),
        compound_index("index_card_codes_by_type_created", CardCodes::Table, CardCodes::TypeId, CardCodes::CreatedAt),
        index("index_card_codes_by_batch_no", CardCodes::Table, CardCodes::BatchNo, false),
        index("index_card_codes_by_expires_at", CardCodes::Table, CardCodes::ExpiresAt, false),
        index("index_card_codes_by_used_at", CardCodes::Table, CardCodes::UsedAt, false),
        index("index_card_codes_by_wallet_transaction", CardCodes::Table, CardCodes::WalletTransactionId, true),
        index("index_recharge_packages_by_status", RechargePackages::Table, RechargePackages::Status, false),
        index("index_recharge_orders_by_order_no", RechargeOrders::Table, RechargeOrders::OrderNo, true),
        compound_index(
            "index_recharge_orders_by_status_created",
            RechargeOrders::Table,
            RechargeOrders::Status,
            RechargeOrders::CreatedAt,
        ),
        index("index_recharge_orders_by_paid_at", RechargeOrders::Table, RechargeOrders::PaidAt, false),
        compound_index(
            "index_recharge_orders_by_user_created",
            RechargeOrders::Table,
            RechargeOrders::UserId,
            RechargeOrders::CreatedAt,
        ),
        recharge_orders_provider_trade_unique_index(),
        index(
            "index_affiliate_commissions_by_order",
            AffiliateCommissions::Table,
            AffiliateCommissions::RechargeOrderId,
            true,
        ),
        compound_index(
            "index_affiliate_commissions_by_referrer_created",
            AffiliateCommissions::Table,
            AffiliateCommissions::ReferrerUserId,
            AffiliateCommissions::CreatedAt,
        ),
        compound_index(
            "index_affiliate_commissions_by_referred_created",
            AffiliateCommissions::Table,
            AffiliateCommissions::ReferredUserId,
            AffiliateCommissions::CreatedAt,
        ),
        compound_index(
            "index_payment_callback_records_by_received",
            PaymentCallbackRecords::Table,
            PaymentCallbackRecords::PaymentChannelCode,
            PaymentCallbackRecords::ReceivedAt,
        ),
        compound_index(
            "index_payment_callback_records_by_order",
            PaymentCallbackRecords::Table,
            PaymentCallbackRecords::OrderNo,
            PaymentCallbackRecords::ReceivedAt,
        ),
        index("index_global_models_by_name", GlobalModels::Table, GlobalModels::Name, true),
        index("index_global_models_by_usage_count", GlobalModels::Table, GlobalModels::UsageCount, false),
        index("index_providers_by_name", Providers::Table, Providers::Name, true),
        index("index_providers_by_active", Providers::Table, Providers::IsActive, false),
        index(
            "index_provider_endpoints_by_provider",
            ProviderEndpoints::Table,
            ProviderEndpoints::ProviderId,
            false,
        ),
        index(
            "index_provider_endpoints_by_format",
            ProviderEndpoints::Table,
            ProviderEndpoints::ApiFormat,
            false,
        ),
        index(
            "index_provider_api_keys_by_provider",
            ProviderApiKeys::Table,
            ProviderApiKeys::ProviderId,
            false,
        ),
        index("index_provider_models_by_provider", ProviderModels::Table, ProviderModels::ProviderId, false),
        index(
            "index_provider_models_by_global_model",
            ProviderModels::Table,
            ProviderModels::GlobalModelId,
            false,
        ),
        provider_models_unique_index(),
        provider_key_model_mappings_unique_index(),
        provider_model_costs_unique_index(),
        quick_import_sources_provider_unique_index(),
        quick_import_keys_key_unique_index(),
        index(
            "index_provider_model_costs_by_provider",
            ProviderModelCosts::Table,
            ProviderModelCosts::ProviderId,
            false,
        ),
        index("index_provider_model_costs_by_key", ProviderModelCosts::Table, ProviderModelCosts::KeyId, false),
        index(
            "index_provider_quick_import_keys_by_source",
            ProviderQuickImportKeys::Table,
            ProviderQuickImportKeys::SourceId,
            false,
        ),
        index(
            "index_provider_key_model_mappings_by_key",
            ProviderKeyModelMappings::Table,
            ProviderKeyModelMappings::KeyId,
            false,
        ),
        index(
            "index_provider_key_model_mappings_by_provider_model",
            ProviderKeyModelMappings::Table,
            ProviderKeyModelMappings::ProviderModelId,
            false,
        ),
        index(
            "index_provider_quick_import_sync_events_by_created",
            ProviderQuickImportSyncEvents::Table,
            ProviderQuickImportSyncEvents::CreatedAt,
            false,
        ),
        billing_rules_global_model_unique_index(),
        billing_rules_model_unique_index(),
        dimension_collectors_enabled_unique_index(),
        compound_index(
            "index_dimension_collectors_by_scope",
            DimensionCollectors::Table,
            DimensionCollectors::ApiFormat,
            DimensionCollectors::TaskType,
        ),
        index(
            "index_provider_cooldowns_by_until",
            ProviderCooldowns::Table,
            ProviderCooldowns::CooldownUntil,
            false,
        ),
        index(
            "index_provider_cooldowns_by_status",
            ProviderCooldowns::Table,
            ProviderCooldowns::StatusCode,
            false,
        ),
        index(
            "index_provider_cooldown_events_by_triggered",
            ProviderCooldownEvents::Table,
            ProviderCooldownEvents::TriggeredAt,
            false,
        ),
        compound_index(
            "index_provider_cooldown_events_by_provider_triggered",
            ProviderCooldownEvents::Table,
            ProviderCooldownEvents::ProviderId,
            ProviderCooldownEvents::TriggeredAt,
        ),
        index("index_billing_groups_by_active", BillingGroups::Table, BillingGroups::IsActive, false),
        index("index_api_tokens_by_hash", ApiTokens::Table, ApiTokens::TokenHash, true),
        index("index_api_tokens_by_user_id", ApiTokens::Table, ApiTokens::UserId, false),
        index("index_api_tokens_by_token_type", ApiTokens::Table, ApiTokens::TokenType, false),
        index("index_api_tokens_by_group_code", ApiTokens::Table, ApiTokens::GroupCode, false),
        billing_group_models_unique_index(),
        index(
            "index_billing_group_models_by_group",
            BillingGroupModels::Table,
            BillingGroupModels::GroupCode,
            false,
        ),
        billing_group_providers_unique_index(),
        index(
            "index_billing_group_providers_by_group",
            BillingGroupProviders::Table,
            BillingGroupProviders::GroupCode,
            false,
        ),
        billing_group_provider_keys_unique_index(),
        index(
            "index_billing_group_provider_keys_by_group",
            BillingGroupProviderKeys::Table,
            BillingGroupProviderKeys::GroupCode,
            false,
        ),
        index(
            "index_billing_group_provider_keys_by_key",
            BillingGroupProviderKeys::Table,
            BillingGroupProviderKeys::ProviderKeyId,
            false,
        ),
        billing_group_user_groups_unique_index(),
        index(
            "index_billing_group_user_groups_by_billing_group",
            BillingGroupUserGroups::Table,
            BillingGroupUserGroups::BillingGroupCode,
            false,
        ),
        index(
            "index_billing_group_user_groups_by_user_group",
            BillingGroupUserGroups::Table,
            BillingGroupUserGroups::UserGroupCode,
            false,
        ),
        index("index_request_records_by_created_at", RequestRecords::Table, RequestRecords::CreatedAt, false),
        compound_index(
            "index_request_records_by_user_created",
            RequestRecords::Table,
            RequestRecords::UserIdSnapshot,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_token_created",
            RequestRecords::Table,
            RequestRecords::TokenId,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_status_created",
            RequestRecords::Table,
            RequestRecords::Status,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_model_created",
            RequestRecords::Table,
            RequestRecords::GlobalModelId,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_provider_created",
            RequestRecords::Table,
            RequestRecords::ProviderId,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_client_format_created",
            RequestRecords::Table,
            RequestRecords::ClientApiFormat,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_provider_format_created",
            RequestRecords::Table,
            RequestRecords::ProviderApiFormat,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_stream_created",
            RequestRecords::Table,
            RequestRecords::IsStream,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_request_records_by_client_error_created",
            RequestRecords::Table,
            RequestRecords::ClientErrorType,
            RequestRecords::CreatedAt,
        ),
        compound_index(
            "index_dashboard_user_usage_buckets_by_bucket",
            DashboardUserUsageBuckets::Table,
            DashboardUserUsageBuckets::BucketGranularity,
            DashboardUserUsageBuckets::BucketStartedAt,
        ),
        dashboard_user_usage_buckets_user_bucket_index(),
        compound_index(
            "index_dashboard_cost_analysis_buckets_by_dimension",
            DashboardCostAnalysisBuckets::Table,
            DashboardCostAnalysisBuckets::DimensionKind,
            DashboardCostAnalysisBuckets::BucketStartedAt,
        ),
        index(
            "index_request_candidates_by_request",
            RequestCandidates::Table,
            RequestCandidates::RequestId,
            false,
        ),
        index(
            "index_request_candidates_by_created_at",
            RequestCandidates::Table,
            RequestCandidates::CreatedAt,
            false,
        ),
        compound_index(
            "index_request_candidates_by_provider_created",
            RequestCandidates::Table,
            RequestCandidates::ProviderId,
            RequestCandidates::CreatedAt,
        ),
        compound_index(
            "index_request_candidates_by_model_created",
            RequestCandidates::Table,
            RequestCandidates::GlobalModelId,
            RequestCandidates::CreatedAt,
        ),
        compound_index(
            "index_request_candidates_by_status_created",
            RequestCandidates::Table,
            RequestCandidates::Status,
            RequestCandidates::CreatedAt,
        ),
        compound_index(
            "index_scheduled_tasks_by_due_claim",
            ScheduledTasks::Table,
            ScheduledTasks::Enabled,
            ScheduledTasks::NextRunAt,
        ),
        compound_index(
            "index_scheduled_task_runs_by_task_started",
            ScheduledTaskRuns::Table,
            ScheduledTaskRuns::TaskCode,
            ScheduledTaskRuns::StartedAt,
        ),
        compound_index(
            "index_scheduled_task_runs_by_status_started",
            ScheduledTaskRuns::Table,
            ScheduledTaskRuns::Status,
            ScheduledTaskRuns::StartedAt,
        ),
        index(
            "index_translation_entries_by_lang",
            TranslationEntries::Table,
            TranslationEntries::LangCode,
            false,
        ),
        translation_entry_unique_index(),
        index("index_announcements_by_enabled", Announcements::Table, Announcements::Enabled, false),
        compound_index(
            "index_announcements_by_pinned_priority",
            Announcements::Table,
            Announcements::Pinned,
            Announcements::Priority,
        ),
        index("index_support_tickets_by_user", SupportTickets::Table, SupportTickets::UserId, false),
        compound_index(
            "index_support_tickets_by_status_updated",
            SupportTickets::Table,
            SupportTickets::Status,
            SupportTickets::UpdatedAt,
        ),
        index(
            "index_support_ticket_messages_by_ticket",
            SupportTicketMessages::Table,
            SupportTicketMessages::TicketId,
            false,
        ),
        index(
            "index_support_ticket_email_events_by_ticket",
            SupportTicketEmailEvents::Table,
            SupportTicketEmailEvents::TicketId,
            false,
        ),
        notification_states_unique_index(),
        performance_monitoring_bucket_unique_index(),
        compound_index(
            "index_performance_monitoring_snapshots_by_bucket",
            PerformanceMonitoringSnapshots::Table,
            PerformanceMonitoringSnapshots::BucketGranularity,
            PerformanceMonitoringSnapshots::BucketStartedAt,
        ),
        compound_index(
            "index_model_status_checks_by_enabled_due",
            ModelStatusChecks::Table,
            ModelStatusChecks::Enabled,
            ModelStatusChecks::NextDueAt,
        ),
        index(
            "index_model_status_checks_by_token",
            ModelStatusChecks::Table,
            ModelStatusChecks::ApiTokenId,
            false,
        ),
        compound_index(
            "index_model_status_runs_by_check_checked",
            ModelStatusCheckRuns::Table,
            ModelStatusCheckRuns::CheckId,
            ModelStatusCheckRuns::CheckedAt,
        ),
        index(
            "index_model_status_runs_by_checked",
            ModelStatusCheckRuns::Table,
            ModelStatusCheckRuns::CheckedAt,
            false,
        ),
        model_status_hourly_stats_unique_index(),
        compound_index(
            "index_model_status_hourly_stats_by_check_bucket",
            ModelStatusCheckHourlyStats::Table,
            ModelStatusCheckHourlyStats::CheckId,
            ModelStatusCheckHourlyStats::BucketStartedAt,
        ),
    ];
    indices.extend(super::provider_key_group_indices::provider_key_group_indices());
    indices
}

fn provider_models_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_provider_models_unique")
        .table(ProviderModels::Table)
        .col(ProviderModels::ProviderId)
        .col(ProviderModels::GlobalModelId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn provider_model_costs_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_provider_model_costs_unique")
        .table(ProviderModelCosts::Table)
        .col(ProviderModelCosts::KeyId)
        .col(ProviderModelCosts::ProviderModelId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn provider_key_model_mappings_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_provider_key_model_mappings_unique")
        .table(ProviderKeyModelMappings::Table)
        .col(ProviderKeyModelMappings::KeyId)
        .col(ProviderKeyModelMappings::ProviderModelId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn quick_import_sources_provider_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_provider_quick_import_sources_provider_unique")
        .table(ProviderQuickImportSources::Table)
        .col(ProviderQuickImportSources::ProviderId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn quick_import_keys_key_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_provider_quick_import_keys_key_unique")
        .table(ProviderQuickImportKeys::Table)
        .col(ProviderQuickImportKeys::KeyId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn billing_rules_global_model_unique_index() -> IndexCreateStatement {
    let mut index = Index::create();
    index
        .name("uq_billing_rules_global_model_task")
        .table(BillingRules::Table)
        .col(BillingRules::GlobalModelId)
        .col(BillingRules::TaskType)
        .unique()
        .if_not_exists()
        .cond_where(Expr::cust("is_enabled = TRUE AND global_model_id IS NOT NULL"));
    index.to_owned()
}

fn recharge_orders_provider_trade_unique_index() -> IndexCreateStatement {
    let mut index = Index::create();
    index
        .name("index_recharge_orders_unique_provider_trade")
        .table(RechargeOrders::Table)
        .col(RechargeOrders::PaymentChannelCode)
        .col(RechargeOrders::ProviderTradeNo)
        .unique()
        .if_not_exists()
        .cond_where(Expr::cust("provider_trade_no IS NOT NULL"));
    index.to_owned()
}

fn billing_rules_model_unique_index() -> IndexCreateStatement {
    let mut index = Index::create();
    index
        .name("uq_billing_rules_model_task")
        .table(BillingRules::Table)
        .col(BillingRules::ModelId)
        .col(BillingRules::TaskType)
        .unique()
        .if_not_exists()
        .cond_where(Expr::cust("is_enabled = TRUE AND model_id IS NOT NULL"));
    index.to_owned()
}

fn dimension_collectors_enabled_unique_index() -> IndexCreateStatement {
    let mut index = Index::create();
    index
        .name("uq_dimension_collectors_enabled")
        .table(DimensionCollectors::Table)
        .col(DimensionCollectors::ApiFormat)
        .col(DimensionCollectors::TaskType)
        .col(DimensionCollectors::DimensionName)
        .col(DimensionCollectors::Priority)
        .unique()
        .if_not_exists()
        .cond_where(Expr::cust("is_enabled = TRUE"));
    index.to_owned()
}

fn billing_group_models_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_billing_group_models_unique")
        .table(BillingGroupModels::Table)
        .col(BillingGroupModels::GroupCode)
        .col(BillingGroupModels::GlobalModelId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn billing_group_providers_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_billing_group_providers_unique")
        .table(BillingGroupProviders::Table)
        .col(BillingGroupProviders::GroupCode)
        .col(BillingGroupProviders::ProviderId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn billing_group_provider_keys_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_billing_group_provider_keys_unique")
        .table(BillingGroupProviderKeys::Table)
        .col(BillingGroupProviderKeys::GroupCode)
        .col(BillingGroupProviderKeys::ProviderKeyId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn billing_group_user_groups_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_billing_group_user_groups_unique")
        .table(BillingGroupUserGroups::Table)
        .col(BillingGroupUserGroups::BillingGroupCode)
        .col(BillingGroupUserGroups::UserGroupCode)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn user_group_memberships_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_user_group_memberships_unique")
        .table(UserGroupMemberships::Table)
        .col(UserGroupMemberships::UserId)
        .col(UserGroupMemberships::UserGroupCode)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn translation_entry_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_translation_entries_unique_key")
        .table(TranslationEntries::Table)
        .col(TranslationEntries::Namespace)
        .col(TranslationEntries::GroupKey)
        .col(TranslationEntries::ItemKey)
        .col(TranslationEntries::LangCode)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn notification_states_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_notification_states_unique_source")
        .table(NotificationStates::Table)
        .col(NotificationStates::UserId)
        .col(NotificationStates::SourceType)
        .col(NotificationStates::SourceId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn performance_monitoring_bucket_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_performance_monitoring_snapshots_unique_bucket")
        .table(PerformanceMonitoringSnapshots::Table)
        .col(PerformanceMonitoringSnapshots::BucketGranularity)
        .col(PerformanceMonitoringSnapshots::BucketStartedAt)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn dashboard_user_usage_buckets_user_bucket_index() -> IndexCreateStatement {
    Index::create()
        .name("index_dashboard_user_usage_buckets_by_user_bucket")
        .table(DashboardUserUsageBuckets::Table)
        .col(DashboardUserUsageBuckets::UserId)
        .col(DashboardUserUsageBuckets::BucketGranularity)
        .col(DashboardUserUsageBuckets::BucketStartedAt)
        .if_not_exists()
        .to_owned()
}

fn model_status_hourly_stats_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_model_status_hourly_stats_unique")
        .table(ModelStatusCheckHourlyStats::Table)
        .col(ModelStatusCheckHourlyStats::CheckId)
        .col(ModelStatusCheckHourlyStats::BucketStartedAt)
        .unique()
        .if_not_exists()
        .to_owned()
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

fn compound_index<T, C1, C2>(name: &str, table: T, first: C1, second: C2) -> IndexCreateStatement
where
    T: Iden + 'static,
    C1: Iden + 'static,
    C2: Iden + 'static,
{
    Index::create().name(name).table(table).col(first).col(second).if_not_exists().to_owned()
}

fn compound_index_unique<T, C1, C2>(name: &str, table: T, first: C1, second: C2) -> IndexCreateStatement
where
    T: Iden + 'static,
    C1: Iden + 'static,
    C2: Iden + 'static,
{
    Index::create()
        .name(name)
        .table(table)
        .col(first)
        .col(second)
        .unique()
        .if_not_exists()
        .to_owned()
}
