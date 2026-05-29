use sea_orm_migration::{prelude::*, schema::*};

use super::{card_code_tables, domain_tables, iden::*, operations_tables, recharge_tables, request_candidate_tables, wallet_tables};

pub(super) fn baseline_tables() -> Vec<TableCreateStatement> {
    let mut tables = vec![
        user_groups_table(),
        users_table(),
        user_identities_table(),
        user_password_reset_tokens_table(),
        roles_table(),
        api_permissions_table(),
        menu_sections_table(),
        menu_items_table(),
        menu_api_permissions_table(),
        role_menu_permissions_table(),
        role_api_permissions_table(),
        wallet_tables::wallets_table(),
        wallet_tables::wallet_transactions_table(),
        card_code_tables::card_code_types_table(),
        card_code_tables::card_codes_table(),
        global_models_table(),
    ];
    tables.extend(recharge_tables::recharge_tables());
    tables.extend(domain_tables::domain_tables());
    tables.push(request_candidate_tables::dashboard_user_usage_buckets_table());
    tables.extend(operations_tables::operations_tables());
    tables
}

fn users_table() -> TableCreateStatement {
    let mut user_group_fk = user_group_fk();
    Table::create()
        .table(Users::Table)
        .if_not_exists()
        .col(string_len(Users::Id, 36).primary_key())
        .col(string_len(Users::Username, 100))
        .col(string_len_null(Users::PasswordHash, 255))
        .col(string_len(Users::Email, 255))
        .col(string_len(Users::Role, 100))
        .col(string_len(Users::GroupCode, 64))
        .col(boolean(Users::IsActive))
        .col(boolean(Users::IsDeleted))
        .col(text(Users::AllowedModelIds).default("[]"))
        .col(text(Users::AllowedProviderIds).default("[]"))
        .col(timestamp_tz(Users::CreatedAt))
        .col(timestamp_tz(Users::UpdatedAt))
        .col(timestamp_tz_null(Users::LastLoginAt))
        .col(string_len(Users::AuthSource, 50))
        .col(boolean(Users::EmailVerified))
        .col(big_integer_null(Users::RateLimitRpm))
        .col(string_len(Users::QuotaMode, 20).default("wallet"))
        .foreign_key(&mut user_group_fk)
        .to_owned()
}

fn user_identities_table() -> TableCreateStatement {
    let mut user_fk = user_identity_user_fk();
    Table::create()
        .table(UserIdentities::Table)
        .if_not_exists()
        .col(string_len(UserIdentities::Id, 36).primary_key())
        .col(string_len(UserIdentities::UserId, 36))
        .col(string_len(UserIdentities::Provider, 20))
        .col(string_len(UserIdentities::ProviderSubject, 255))
        .col(string_len_null(UserIdentities::Email, 255))
        .col(boolean(UserIdentities::EmailVerified))
        .col(string_len_null(UserIdentities::DisplayName, 255))
        .col(string_len_null(UserIdentities::AvatarUrl, 1024))
        .col(text(UserIdentities::MetadataJson).default("{}"))
        .col(timestamp_tz(UserIdentities::CreatedAt))
        .col(timestamp_tz(UserIdentities::UpdatedAt))
        .col(timestamp_tz_null(UserIdentities::LastLoginAt))
        .foreign_key(&mut user_fk)
        .to_owned()
}

fn user_groups_table() -> TableCreateStatement {
    Table::create()
        .table(UserGroups::Table)
        .if_not_exists()
        .col(string_len(UserGroups::Id, 36).primary_key())
        .col(string_len(UserGroups::Code, 64))
        .col(string_len(UserGroups::Name, 100))
        .col(text_null(UserGroups::Description))
        .col(boolean(UserGroups::IsActive))
        .col(boolean(UserGroups::IsSystem))
        .col(big_integer(UserGroups::SortOrder))
        .col(timestamp_tz(UserGroups::CreatedAt))
        .col(timestamp_tz(UserGroups::UpdatedAt))
        .index(Index::create().name("index_user_groups_by_code").col(UserGroups::Code).unique())
        .to_owned()
}

fn user_group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_users_user_group")
        .from(Users::Table, Users::GroupCode)
        .to(UserGroups::Table, UserGroups::Code);
    foreign_key
}

fn user_password_reset_tokens_table() -> TableCreateStatement {
    let mut user_fk = user_password_reset_token_user_fk();
    Table::create()
        .table(UserPasswordResetTokens::Table)
        .if_not_exists()
        .col(string_len(UserPasswordResetTokens::Id, 36).primary_key())
        .col(string_len(UserPasswordResetTokens::UserId, 36))
        .col(string_len(UserPasswordResetTokens::TokenHash, 64))
        .col(timestamp_tz(UserPasswordResetTokens::ExpiresAt))
        .col(timestamp_tz_null(UserPasswordResetTokens::ConsumedAt))
        .col(timestamp_tz(UserPasswordResetTokens::CreatedAt))
        .foreign_key(&mut user_fk)
        .to_owned()
}

fn user_password_reset_token_user_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_user_password_reset_tokens_user")
        .from(UserPasswordResetTokens::Table, UserPasswordResetTokens::UserId)
        .to(Users::Table, Users::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn user_identity_user_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_user_identities_user")
        .from(UserIdentities::Table, UserIdentities::UserId)
        .to(Users::Table, Users::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn roles_table() -> TableCreateStatement {
    Table::create()
        .table(Roles::Table)
        .if_not_exists()
        .col(text(Roles::Code).primary_key())
        .col(text(Roles::Name))
        .col(text(Roles::Description))
        .col(boolean(Roles::Enabled))
        .col(boolean(Roles::System))
        .col(big_integer(Roles::SortOrder))
        .col(timestamp_tz(Roles::CreatedAt))
        .col(timestamp_tz(Roles::UpdatedAt))
        .to_owned()
}

fn api_permissions_table() -> TableCreateStatement {
    Table::create()
        .table(ApiPermissions::Table)
        .if_not_exists()
        .col(string_len(ApiPermissions::Id, 36).primary_key())
        .col(text(ApiPermissions::Code))
        .col(text(ApiPermissions::Method))
        .col(text(ApiPermissions::PathPattern))
        .col(text(ApiPermissions::Name))
        .col(boolean(ApiPermissions::Enabled))
        .col(boolean(ApiPermissions::System))
        .col(timestamp_tz(ApiPermissions::CreatedAt))
        .col(timestamp_tz(ApiPermissions::UpdatedAt))
        .to_owned()
}

fn menu_sections_table() -> TableCreateStatement {
    Table::create()
        .table(MenuSections::Table)
        .if_not_exists()
        .col(string_len(MenuSections::Id, 36).primary_key())
        .col(text(MenuSections::Code))
        .col(text(MenuSections::Subheader))
        .col(big_integer(MenuSections::SortOrder))
        .col(boolean(MenuSections::Enabled))
        .col(timestamp_tz(MenuSections::CreatedAt))
        .col(timestamp_tz(MenuSections::UpdatedAt))
        .to_owned()
}

fn menu_items_table() -> TableCreateStatement {
    Table::create()
        .table(MenuItems::Table)
        .if_not_exists()
        .col(string_len(MenuItems::Id, 36).primary_key())
        .col(string_len(MenuItems::SectionId, 36))
        .col(string_len_null(MenuItems::ParentId, 36))
        .col(text(MenuItems::Code))
        .col(text(MenuItems::Title))
        .col(text(MenuItems::RoutePath))
        .col(text_null(MenuItems::Icon))
        .col(text_null(MenuItems::Caption))
        .col(boolean(MenuItems::DeepMatch))
        .col(big_integer(MenuItems::SortOrder))
        .col(boolean(MenuItems::Enabled))
        .col(timestamp_tz(MenuItems::CreatedAt))
        .col(timestamp_tz(MenuItems::UpdatedAt))
        .to_owned()
}

fn menu_api_permissions_table() -> TableCreateStatement {
    Table::create()
        .table(MenuApiPermissions::Table)
        .if_not_exists()
        .col(string_len(MenuApiPermissions::MenuItemId, 36))
        .col(string_len(MenuApiPermissions::ApiPermissionId, 36))
        .col(timestamp_tz(MenuApiPermissions::CreatedAt))
        .col(timestamp_tz(MenuApiPermissions::UpdatedAt))
        .primary_key(
            Index::create()
                .name("pk_menu_api_permissions")
                .col(MenuApiPermissions::MenuItemId)
                .col(MenuApiPermissions::ApiPermissionId),
        )
        .to_owned()
}

fn role_menu_permissions_table() -> TableCreateStatement {
    Table::create()
        .table(RoleMenuPermissions::Table)
        .if_not_exists()
        .col(text(RoleMenuPermissions::RoleCode))
        .col(string_len(RoleMenuPermissions::MenuItemId, 36))
        .col(timestamp_tz(RoleMenuPermissions::CreatedAt))
        .col(timestamp_tz(RoleMenuPermissions::UpdatedAt))
        .primary_key(
            Index::create()
                .name("pk_role_menu_permissions")
                .col(RoleMenuPermissions::RoleCode)
                .col(RoleMenuPermissions::MenuItemId),
        )
        .to_owned()
}

fn role_api_permissions_table() -> TableCreateStatement {
    Table::create()
        .table(RoleApiPermissions::Table)
        .if_not_exists()
        .col(text(RoleApiPermissions::RoleCode))
        .col(string_len(RoleApiPermissions::ApiPermissionId, 36))
        .col(timestamp_tz(RoleApiPermissions::CreatedAt))
        .col(timestamp_tz(RoleApiPermissions::UpdatedAt))
        .primary_key(
            Index::create()
                .name("pk_role_api_permissions")
                .col(RoleApiPermissions::RoleCode)
                .col(RoleApiPermissions::ApiPermissionId),
        )
        .to_owned()
}

fn global_models_table() -> TableCreateStatement {
    Table::create()
        .table(GlobalModels::Table)
        .if_not_exists()
        .col(string_len(GlobalModels::Id, 36).primary_key())
        .col(string_len(GlobalModels::Name, 100))
        .col(string_len(GlobalModels::DisplayName, 100))
        .col(decimal_len_null(GlobalModels::DefaultPricePerRequest, 20, 8))
        .col(text(GlobalModels::DefaultTieredPricing))
        .col(text_null(GlobalModels::SupportedCapabilities))
        .col(text_null(GlobalModels::Config))
        .col(boolean(GlobalModels::IsActive))
        .col(big_integer(GlobalModels::UsageCount))
        .col(timestamp_tz(GlobalModels::CreatedAt))
        .col(timestamp_tz(GlobalModels::UpdatedAt))
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
