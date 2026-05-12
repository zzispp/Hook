use sea_orm_migration::prelude::*;

use super::iden::*;

pub(super) fn baseline_indices() -> Vec<IndexCreateStatement> {
    vec![
        index("index_users_by_username", Users::Table, Users::Username, true),
        index("index_users_by_email", Users::Table, Users::Email, true),
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
        index(
            "index_translation_entries_by_lang",
            TranslationEntries::Table,
            TranslationEntries::LangCode,
            false,
        ),
        translation_entry_unique_index(),
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
