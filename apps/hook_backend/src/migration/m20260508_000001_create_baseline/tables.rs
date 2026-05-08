use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

pub(super) fn baseline_tables() -> Vec<TableCreateStatement> {
    vec![
        users_table(),
        roles_table(),
        api_permissions_table(),
        menu_sections_table(),
        menu_items_table(),
        role_api_permissions_table(),
        role_menu_permissions_table(),
        global_models_table(),
        models_table(),
    ]
}

fn users_table() -> TableCreateStatement {
    Table::create()
        .table(Users::Table)
        .if_not_exists()
        .col(string_len(Users::Id, 36).primary_key())
        .col(string_len(Users::Username, 100))
        .col(string_len(Users::PasswordHash, 255))
        .col(string_len(Users::Email, 255))
        .col(string_len(Users::Role, 100))
        .col(boolean(Users::IsActive))
        .col(boolean(Users::IsDeleted))
        .col(timestamp_tz(Users::CreatedAt))
        .col(timestamp_tz(Users::UpdatedAt))
        .col(timestamp_tz_null(Users::LastLoginAt))
        .col(string_len(Users::AuthSource, 50))
        .col(boolean(Users::EmailVerified))
        .to_owned()
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
        .col(text(ApiPermissions::Group))
        .col(boolean(ApiPermissions::Enabled))
        .col(boolean(ApiPermissions::System))
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
        .to_owned()
}

fn role_api_permissions_table() -> TableCreateStatement {
    Table::create()
        .table(RoleApiPermissions::Table)
        .if_not_exists()
        .col(text(RoleApiPermissions::RoleCode))
        .col(string_len(RoleApiPermissions::ApiPermissionId, 36))
        .primary_key(
            Index::create()
                .name("pk_role_api_permissions")
                .col(RoleApiPermissions::RoleCode)
                .col(RoleApiPermissions::ApiPermissionId),
        )
        .to_owned()
}

fn role_menu_permissions_table() -> TableCreateStatement {
    Table::create()
        .table(RoleMenuPermissions::Table)
        .if_not_exists()
        .col(text(RoleMenuPermissions::RoleCode))
        .col(string_len(RoleMenuPermissions::MenuItemId, 36))
        .primary_key(
            Index::create()
                .name("pk_role_menu_permissions")
                .col(RoleMenuPermissions::RoleCode)
                .col(RoleMenuPermissions::MenuItemId),
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

fn models_table() -> TableCreateStatement {
    Table::create()
        .table(Models::Table)
        .if_not_exists()
        .col(string_len(Models::Id, 36).primary_key())
        .col(string_len(Models::ProviderId, 36))
        .col(string_len(Models::GlobalModelId, 36))
        .col(string_len(Models::ProviderModelName, 200))
        .col(text_null(Models::ProviderModelMappings))
        .col(decimal_len_null(Models::PricePerRequest, 20, 8))
        .col(text_null(Models::TieredPricing))
        .col(boolean_null(Models::SupportsVision))
        .col(boolean_null(Models::SupportsFunctionCalling))
        .col(boolean_null(Models::SupportsStreaming))
        .col(boolean_null(Models::SupportsExtendedThinking))
        .col(boolean_null(Models::SupportsImageGeneration))
        .col(boolean(Models::IsActive))
        .col(boolean(Models::IsAvailable))
        .col(text_null(Models::Config))
        .col(timestamp_tz(Models::CreatedAt))
        .col(timestamp_tz(Models::UpdatedAt))
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
