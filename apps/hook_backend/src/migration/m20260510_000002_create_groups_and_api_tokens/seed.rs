use sea_orm_migration::prelude::*;

use super::{super::defaults, definitions::*, iden::*};

pub(super) async fn seed_defaults(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_default_group(manager).await?;
    seed_api_permissions(manager).await?;
    seed_menu_items(manager).await?;
    seed_menu_api_bindings(manager).await?;
    seed_role_menu_bindings(manager).await
}

pub(super) async fn remove_defaults(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    delete_role_menu_bindings(manager).await?;
    delete_menu_api_bindings(manager).await?;
    delete_menu_items(manager).await?;
    delete_api_permissions(manager).await?;
    delete_default_group(manager).await
}

async fn seed_default_group(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(BillingGroups::Table)
                .columns([
                    BillingGroups::Id,
                    BillingGroups::Code,
                    BillingGroups::Name,
                    BillingGroups::Description,
                    BillingGroups::BillingMultiplier,
                    BillingGroups::IsActive,
                    BillingGroups::IsSystem,
                    BillingGroups::SortOrder,
                    BillingGroups::CreatedAt,
                    BillingGroups::UpdatedAt,
                ])
                .values_panic([
                    DEFAULT_GROUP_ID.into(),
                    DEFAULT_GROUP_CODE.into(),
                    "System Group".into(),
                    Some("Built-in billing group used when a token does not choose a group").into(),
                    1.into(),
                    true.into(),
                    true.into(),
                    0.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .on_conflict(OnConflict::column(BillingGroups::Code).do_nothing().to_owned())
                .to_owned(),
        )
        .await
}

async fn seed_api_permissions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(ApiPermissions::Table).columns([
        ApiPermissions::Id,
        ApiPermissions::Code,
        ApiPermissions::Method,
        ApiPermissions::PathPattern,
        ApiPermissions::Name,
        ApiPermissions::Group,
        ApiPermissions::Enabled,
        ApiPermissions::System,
        ApiPermissions::CreatedAt,
        ApiPermissions::UpdatedAt,
    ]);
    for (index, api) in API_DEFINITIONS.iter().enumerate() {
        insert.values_panic(api_values(index, api));
    }
    manager
        .execute(insert.on_conflict(OnConflict::column(ApiPermissions::Code).do_nothing().to_owned()).to_owned())
        .await
}

async fn seed_menu_items(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(MenuItems::Table).columns([
        MenuItems::Id,
        MenuItems::SectionId,
        MenuItems::ParentId,
        MenuItems::Code,
        MenuItems::Title,
        MenuItems::RoutePath,
        MenuItems::Icon,
        MenuItems::Caption,
        MenuItems::DeepMatch,
        MenuItems::SortOrder,
        MenuItems::Enabled,
        MenuItems::CreatedAt,
        MenuItems::UpdatedAt,
    ]);
    for menu in MENU_DEFINITIONS {
        insert.values_panic(menu_values(menu));
    }
    manager
        .execute(insert.on_conflict(OnConflict::column(MenuItems::Code).do_nothing().to_owned()).to_owned())
        .await
}

async fn seed_menu_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(MenuApiPermissions::Table).columns([
        MenuApiPermissions::MenuItemId,
        MenuApiPermissions::ApiPermissionId,
        MenuApiPermissions::CreatedAt,
        MenuApiPermissions::UpdatedAt,
    ]);
    for binding in MENU_API_BINDINGS {
        for code in binding.api_codes {
            insert.values_panic(binding_values(binding.menu_id, api_id_for_code(code)?));
        }
    }
    manager
        .execute(
            insert
                .on_conflict(
                    OnConflict::columns([MenuApiPermissions::MenuItemId, MenuApiPermissions::ApiPermissionId])
                        .do_nothing()
                        .to_owned(),
                )
                .to_owned(),
        )
        .await
}

async fn seed_role_menu_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(RoleMenuPermissions::Table).columns([
        RoleMenuPermissions::RoleCode,
        RoleMenuPermissions::MenuItemId,
        RoleMenuPermissions::CreatedAt,
        RoleMenuPermissions::UpdatedAt,
    ]);
    insert.values_panic(role_menu_values(USER_ROLE, API_TOKEN_MENU_ID));
    insert.values_panic(role_menu_values(ADMIN_ROLE, ADMIN_GROUP_MENU_ID));
    manager
        .execute(
            insert
                .on_conflict(
                    OnConflict::columns([RoleMenuPermissions::RoleCode, RoleMenuPermissions::MenuItemId])
                        .do_nothing()
                        .to_owned(),
                )
                .to_owned(),
        )
        .await
}

async fn delete_role_menu_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(RoleMenuPermissions::Table)
                .and_where(Expr::col(RoleMenuPermissions::MenuItemId).is_in([API_TOKEN_MENU_ID, ADMIN_GROUP_MENU_ID]))
                .to_owned(),
        )
        .await
}

async fn delete_menu_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(MenuApiPermissions::Table)
                .and_where(Expr::col(MenuApiPermissions::MenuItemId).is_in([API_TOKEN_MENU_ID, ADMIN_GROUP_MENU_ID]))
                .to_owned(),
        )
        .await
}

async fn delete_menu_items(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(MenuItems::Table)
                .and_where(Expr::col(MenuItems::Code).is_in(["api_tokens", "admin_groups"]))
                .to_owned(),
        )
        .await
}

async fn delete_api_permissions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let codes = API_DEFINITIONS.iter().map(|api| api.code);
    manager
        .execute(
            Query::delete()
                .from_table(ApiPermissions::Table)
                .and_where(Expr::col(ApiPermissions::Code).is_in(codes))
                .to_owned(),
        )
        .await
}

async fn delete_default_group(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(BillingGroups::Table)
                .and_where(Expr::col(BillingGroups::Code).eq(DEFAULT_GROUP_CODE))
                .to_owned(),
        )
        .await
}

fn api_values(index: usize, api: &ApiDefinition) -> [Expr; 10] {
    [
        default_api_id(index).into(),
        api.code.into(),
        api.method.into(),
        api.path_pattern.into(),
        api.name.into(),
        api.group.into(),
        true.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn menu_values(menu: &MenuDefinition) -> [Expr; 13] {
    [
        menu.id.into(),
        menu.section_id.into(),
        Option::<String>::None.into(),
        menu.code.into(),
        menu.title.into(),
        menu.path.into(),
        Some(menu.icon.to_owned()).into(),
        Option::<String>::None.into(),
        true.into(),
        menu.sort_order.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn binding_values(menu_id: &str, api_id: String) -> [Expr; 4] {
    [menu_id.into(), api_id.into(), Expr::current_timestamp(), Expr::current_timestamp()]
}

fn role_menu_values(role_code: &str, menu_id: &str) -> [Expr; 4] {
    [role_code.into(), menu_id.into(), Expr::current_timestamp(), Expr::current_timestamp()]
}

fn api_id_for_code(code: &str) -> Result<String, DbErr> {
    if let Some(index) = API_DEFINITIONS.iter().position(|api| api.code == code) {
        return Ok(default_api_id(index));
    }
    if let Some(index) = defaults::api::position_by_code(code) {
        return Ok(baseline_api_id(index));
    }
    Err(DbErr::Custom(format!("seed api code does not exist: {code}")))
}

fn default_api_id(index: usize) -> String {
    format!("00000000-0000-7000-8000-000000000{:03}", API_BASE_ID + index)
}

fn baseline_api_id(index: usize) -> String {
    format!("00000000-0000-7000-8000-000000000{:03}", 301 + index)
}
