use sea_orm_migration::prelude::*;

use super::iden::*;

pub(super) fn baseline_indices() -> Vec<IndexCreateStatement> {
    vec![
        index("index_users_by_username", Users::Table, Users::Username, true),
        index("index_users_by_email", Users::Table, Users::Email, true),
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
        compound_index(
            "index_recharge_orders_by_user_created",
            RechargeOrders::Table,
            RechargeOrders::UserId,
            RechargeOrders::CreatedAt,
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
    ]
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
