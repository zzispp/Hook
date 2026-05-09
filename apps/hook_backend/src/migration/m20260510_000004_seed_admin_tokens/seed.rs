use sea_orm_migration::prelude::*;

use super::{super::defaults, definitions::*, iden::*};

pub(super) async fn seed_defaults(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_api_permissions(manager).await?;
    seed_menu_item(manager).await?;
    seed_menu_api_bindings(manager).await?;
    seed_role_menu_binding(manager).await
}

pub(super) async fn remove_defaults(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    delete_role_menu_binding(manager).await?;
    delete_menu_api_bindings(manager).await?;
    delete_menu_item(manager).await?;
    delete_api_permissions(manager).await
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

async fn seed_menu_item(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(MenuItems::Table)
                .columns(menu_columns())
                .values_panic(menu_values())
                .on_conflict(OnConflict::column(MenuItems::Code).do_nothing().to_owned())
                .to_owned(),
        )
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
    for code in ADMIN_TOKEN_API_CODES {
        insert.values_panic(binding_values(code)?);
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

async fn seed_role_menu_binding(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(RoleMenuPermissions::Table)
                .columns([
                    RoleMenuPermissions::RoleCode,
                    RoleMenuPermissions::MenuItemId,
                    RoleMenuPermissions::CreatedAt,
                    RoleMenuPermissions::UpdatedAt,
                ])
                .values_panic([
                    ADMIN_ROLE.into(),
                    ADMIN_TOKEN_MENU_ID.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .on_conflict(
                    OnConflict::columns([RoleMenuPermissions::RoleCode, RoleMenuPermissions::MenuItemId])
                        .do_nothing()
                        .to_owned(),
                )
                .to_owned(),
        )
        .await
}

async fn delete_role_menu_binding(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(RoleMenuPermissions::Table)
                .and_where(Expr::col(RoleMenuPermissions::MenuItemId).eq(ADMIN_TOKEN_MENU_ID))
                .to_owned(),
        )
        .await
}

async fn delete_menu_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(MenuApiPermissions::Table)
                .and_where(Expr::col(MenuApiPermissions::MenuItemId).eq(ADMIN_TOKEN_MENU_ID))
                .to_owned(),
        )
        .await
}

async fn delete_menu_item(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(MenuItems::Table)
                .and_where(Expr::col(MenuItems::Code).eq("admin_tokens"))
                .to_owned(),
        )
        .await
}

async fn delete_api_permissions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(ApiPermissions::Table)
                .and_where(Expr::col(ApiPermissions::Code).is_in(API_DEFINITIONS.iter().map(|api| api.code)))
                .to_owned(),
        )
        .await
}

fn menu_columns() -> [MenuItems; 13] {
    [
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
    ]
}

fn menu_values() -> [Expr; 13] {
    [
        ADMIN_TOKEN_MENU_ID.into(),
        SYSTEM_SECTION_ID.into(),
        Option::<String>::None.into(),
        "admin_tokens".into(),
        "Token Management".into(),
        "/dashboard/admin/tokens".into(),
        Some("icon.key".to_owned()).into(),
        Option::<String>::None.into(),
        true.into(),
        55.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
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

fn binding_values(code: &str) -> Result<[Expr; 4], DbErr> {
    Ok([
        ADMIN_TOKEN_MENU_ID.into(),
        api_id_for_code(code)?.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ])
}

fn api_id_for_code(code: &str) -> Result<String, DbErr> {
    if let Some(index) = API_DEFINITIONS.iter().position(|api| api.code == code) {
        return Ok(default_api_id(index));
    }
    if let Some(id) = prior_api_id_for_code(code) {
        return Ok(id.to_owned());
    }
    if let Some(index) = defaults::api::position_by_code(code) {
        return Ok(format!("00000000-0000-7000-8000-000000000{:03}", 301 + index));
    }
    Err(DbErr::Custom(format!("seed api code does not exist: {code}")))
}

fn prior_api_id_for_code(code: &str) -> Option<&'static str> {
    match code {
        "groups_available_read" => Some("00000000-0000-7000-8000-000000000406"),
        _ => None,
    }
}

fn default_api_id(index: usize) -> String {
    format!("00000000-0000-7000-8000-000000000{:03}", API_BASE_ID + index)
}
