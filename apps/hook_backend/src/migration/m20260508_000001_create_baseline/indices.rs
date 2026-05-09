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
        index("index_models_by_provider_id", Models::Table, Models::ProviderId, false),
        index("index_models_by_global_model_id", Models::Table, Models::GlobalModelId, false),
    ]
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
