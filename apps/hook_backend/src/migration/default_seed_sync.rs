use sea_orm_migration::{prelude::*, sea_orm::ConnectionTrait};

use super::defaults;

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    sync_default_api_permissions(manager).await?;
    sync_default_menu_sections(manager).await?;
    sync_default_menu_items(manager).await?;
    sync_menu_api_bindings(manager).await?;
    sync_role_menu_bindings(manager).await
}

async fn sync_default_api_permissions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for (index, definition) in defaults::api::iter_definitions().enumerate() {
        if exists_by_value(manager, ApiPermissions::Table, ApiPermissions::Code, definition.code).await? {
            continue;
        }
        insert_api_permission(manager, index, definition).await?;
    }
    Ok(())
}

async fn insert_api_permission(manager: &SchemaManager<'_>, index: usize, definition: &defaults::api::ApiDefinition) -> Result<(), DbErr> {
    let id = preferred_id_or_generated(manager, ApiPermissions::Table, ApiPermissions::Id, &default_api_id(index)).await?;
    manager
        .execute(
            Query::insert()
                .into_table(ApiPermissions::Table)
                .columns(api_permission_columns())
                .values_panic([
                    id.into(),
                    definition.code.into(),
                    definition.method.into(),
                    definition.path_pattern.into(),
                    definition.name.into(),
                    true.into(),
                    true.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .to_owned(),
        )
        .await?;
    Ok(())
}

fn api_permission_columns() -> [ApiPermissions; 9] {
    [
        ApiPermissions::Id,
        ApiPermissions::Code,
        ApiPermissions::Method,
        ApiPermissions::PathPattern,
        ApiPermissions::Name,
        ApiPermissions::Enabled,
        ApiPermissions::System,
        ApiPermissions::CreatedAt,
        ApiPermissions::UpdatedAt,
    ]
}

async fn sync_default_menu_sections(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for definition in defaults::menu::MENU_SECTIONS {
        if exists_by_value(manager, MenuSections::Table, MenuSections::Code, definition.code).await? {
            continue;
        }
        let id = preferred_id_or_generated(manager, MenuSections::Table, MenuSections::Id, definition.id).await?;
        manager
            .execute(
                Query::insert()
                    .into_table(MenuSections::Table)
                    .columns(menu_section_columns())
                    .values_panic([
                        id.into(),
                        definition.code.into(),
                        definition.subheader.into(),
                        definition.sort_order.into(),
                        true.into(),
                        Expr::current_timestamp(),
                        Expr::current_timestamp(),
                    ])
                    .to_owned(),
            )
            .await?;
    }
    Ok(())
}

fn menu_section_columns() -> [MenuSections; 7] {
    [
        MenuSections::Id,
        MenuSections::Code,
        MenuSections::Subheader,
        MenuSections::SortOrder,
        MenuSections::Enabled,
        MenuSections::CreatedAt,
        MenuSections::UpdatedAt,
    ]
}

async fn sync_default_menu_items(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for definition in defaults::menu::MENU_ITEMS {
        if exists_by_value(manager, MenuItems::Table, MenuItems::Code, definition.code).await? {
            continue;
        }
        let id = preferred_id_or_generated(manager, MenuItems::Table, MenuItems::Id, definition.id).await?;
        let section_id = menu_section_id(manager, definition.section_id).await?;
        manager
            .execute(
                Query::insert()
                    .into_table(MenuItems::Table)
                    .columns(menu_item_columns())
                    .values_panic([
                        id.into(),
                        section_id.into(),
                        Option::<String>::None.into(),
                        definition.code.into(),
                        definition.title.into(),
                        definition.path.into(),
                        Some(definition.icon.to_owned()).into(),
                        Option::<String>::None.into(),
                        definition.deep_match.into(),
                        definition.sort_order.into(),
                        true.into(),
                        Expr::current_timestamp(),
                        Expr::current_timestamp(),
                    ])
                    .to_owned(),
            )
            .await?;
    }
    Ok(())
}

fn menu_item_columns() -> [MenuItems; 13] {
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

async fn sync_menu_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for binding in defaults::MENU_API_BINDINGS {
        let Some(menu_item_id) = id_by_code(manager, MenuItems::Table, MenuItems::Id, MenuItems::Code, binding.menu_code).await? else {
            continue;
        };
        for api_code in binding.api_codes {
            let Some(api_permission_id) = id_by_code(manager, ApiPermissions::Table, ApiPermissions::Id, ApiPermissions::Code, api_code).await? else {
                continue;
            };
            if menu_api_binding_exists(manager, &menu_item_id, &api_permission_id).await? {
                continue;
            }
            insert_menu_api_binding(manager, &menu_item_id, &api_permission_id).await?;
        }
    }
    Ok(())
}

async fn insert_menu_api_binding(manager: &SchemaManager<'_>, menu_item_id: &str, api_permission_id: &str) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(MenuApiPermissions::Table)
                .columns(menu_api_columns())
                .values_panic([
                    menu_item_id.into(),
                    api_permission_id.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .to_owned(),
        )
        .await?;
    Ok(())
}

fn menu_api_columns() -> [MenuApiPermissions; 4] {
    [
        MenuApiPermissions::MenuItemId,
        MenuApiPermissions::ApiPermissionId,
        MenuApiPermissions::CreatedAt,
        MenuApiPermissions::UpdatedAt,
    ]
}

async fn sync_role_menu_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for role_code in [defaults::ADMIN_ROLE, defaults::USER_ROLE] {
        let menu_codes = role_menu_codes(role_code);
        for menu_code in menu_codes {
            let Some(menu_item_id) = id_by_code(manager, MenuItems::Table, MenuItems::Id, MenuItems::Code, menu_code).await? else {
                continue;
            };
            if role_menu_binding_exists(manager, role_code, &menu_item_id).await? {
                continue;
            }
            manager
                .execute(
                    Query::insert()
                        .into_table(RoleMenuPermissions::Table)
                        .columns(role_menu_columns())
                        .values_panic([role_code.into(), menu_item_id.into(), Expr::current_timestamp(), Expr::current_timestamp()])
                        .to_owned(),
                )
                .await?;
        }
    }
    Ok(())
}

fn role_menu_codes(role_code: &str) -> &'static [&'static str] {
    if role_code == defaults::ADMIN_ROLE {
        defaults::ADMIN_MENU_CODES
    } else {
        defaults::USER_MENU_CODES
    }
}

fn role_menu_columns() -> [RoleMenuPermissions; 4] {
    [
        RoleMenuPermissions::RoleCode,
        RoleMenuPermissions::MenuItemId,
        RoleMenuPermissions::CreatedAt,
        RoleMenuPermissions::UpdatedAt,
    ]
}

async fn menu_section_id(manager: &SchemaManager<'_>, preferred_section_id: &str) -> Result<String, DbErr> {
    let section_code = menu_section_code(preferred_section_id)?;
    id_by_code(manager, MenuSections::Table, MenuSections::Id, MenuSections::Code, section_code)
        .await?
        .ok_or_else(|| DbErr::Migration(format!("default menu section is missing after sync: {section_code}")))
}

fn menu_section_code(section_id: &str) -> Result<&'static str, DbErr> {
    defaults::menu::MENU_SECTIONS
        .iter()
        .find(|definition| definition.id == section_id)
        .map(|definition| definition.code)
        .ok_or_else(|| DbErr::Migration(format!("unknown default menu section id: {section_id}")))
}

async fn preferred_id_or_generated<T: Iden + 'static, C: Iden + 'static>(
    manager: &SchemaManager<'_>,
    table: T,
    id_column: C,
    preferred_id: &str,
) -> Result<String, DbErr> {
    if exists_by_value(manager, table, id_column, preferred_id).await? {
        return Ok(uuid::Uuid::now_v7().to_string());
    }
    Ok(preferred_id.to_owned())
}

async fn exists_by_value<T: Iden + 'static, C: Iden + 'static>(manager: &SchemaManager<'_>, table: T, column: C, value: &str) -> Result<bool, DbErr> {
    manager
        .get_connection()
        .query_one(
            &Query::select()
                .expr(Expr::val(1))
                .from(table)
                .and_where(Expr::col(column).eq(value))
                .limit(1)
                .to_owned(),
        )
        .await
        .map(|row| row.is_some())
}

async fn id_by_code<T: Iden + 'static, I: Iden + 'static, C: Iden + 'static>(
    manager: &SchemaManager<'_>,
    table: T,
    id_column: I,
    code_column: C,
    code: &str,
) -> Result<Option<String>, DbErr> {
    let query = Query::select()
        .column(id_column)
        .from(table)
        .and_where(Expr::col(code_column).eq(code))
        .limit(1)
        .to_owned();
    let Some(row) = manager.get_connection().query_one(&query).await? else {
        return Ok(None);
    };
    Ok(row.try_get_by_index::<String>(0).ok())
}

async fn menu_api_binding_exists(manager: &SchemaManager<'_>, menu_item_id: &str, api_permission_id: &str) -> Result<bool, DbErr> {
    manager
        .get_connection()
        .query_one(
            &Query::select()
                .expr(Expr::val(1))
                .from(MenuApiPermissions::Table)
                .and_where(Expr::col(MenuApiPermissions::MenuItemId).eq(menu_item_id))
                .and_where(Expr::col(MenuApiPermissions::ApiPermissionId).eq(api_permission_id))
                .limit(1)
                .to_owned(),
        )
        .await
        .map(|row| row.is_some())
}

async fn role_menu_binding_exists(manager: &SchemaManager<'_>, role_code: &str, menu_item_id: &str) -> Result<bool, DbErr> {
    manager
        .get_connection()
        .query_one(
            &Query::select()
                .expr(Expr::val(1))
                .from(RoleMenuPermissions::Table)
                .and_where(Expr::col(RoleMenuPermissions::RoleCode).eq(role_code))
                .and_where(Expr::col(RoleMenuPermissions::MenuItemId).eq(menu_item_id))
                .limit(1)
                .to_owned(),
        )
        .await
        .map(|row| row.is_some())
}

fn default_api_id(index: usize) -> String {
    format!("00000000-0000-7000-8000-000000000{:03}", 301 + index)
}

#[derive(Clone, Copy, DeriveIden)]
enum ApiPermissions {
    Table,
    Id,
    Code,
    Method,
    PathPattern,
    Name,
    Enabled,
    System,
    CreatedAt,
    UpdatedAt,
}

#[derive(Clone, Copy, DeriveIden)]
enum MenuSections {
    Table,
    Id,
    Code,
    Subheader,
    SortOrder,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(Clone, Copy, DeriveIden)]
enum MenuItems {
    Table,
    Id,
    SectionId,
    ParentId,
    Code,
    Title,
    RoutePath,
    Icon,
    Caption,
    DeepMatch,
    SortOrder,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(Clone, Copy, DeriveIden)]
enum MenuApiPermissions {
    Table,
    MenuItemId,
    ApiPermissionId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Clone, Copy, DeriveIden)]
enum RoleMenuPermissions {
    Table,
    RoleCode,
    MenuItemId,
    CreatedAt,
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use super::menu_section_code;
    use crate::migration::defaults::menu::OPERATIONS_SECTION_ID;

    #[test]
    fn resolves_known_menu_section_code() {
        assert_eq!(menu_section_code(OPERATIONS_SECTION_ID).unwrap(), "operations");
    }

    #[test]
    fn rejects_unknown_menu_section_code() {
        assert!(menu_section_code("missing").is_err());
    }
}
