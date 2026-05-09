use sea_orm_migration::prelude::*;

use super::{super::defaults, iden::*};

pub(super) async fn seed_defaults(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_roles(manager).await?;
    seed_api_permissions(manager).await?;
    seed_menu_sections(manager).await?;
    seed_menu_items(manager).await?;
    seed_bindings(manager).await
}

async fn seed_roles(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let insert = Query::insert()
        .into_table(Roles::Table)
        .columns([
            Roles::Code,
            Roles::Name,
            Roles::Description,
            Roles::Enabled,
            Roles::System,
            Roles::SortOrder,
            Roles::CreatedAt,
            Roles::UpdatedAt,
        ])
        .values_panic(role_values(RoleSeed {
            code: defaults::ADMIN_ROLE,
            name: "Administrator",
            description: "Built-in administrator role",
            sort_order: 0,
        }))
        .values_panic(role_values(RoleSeed {
            code: defaults::USER_ROLE,
            name: "User",
            description: "Default signed-up user role",
            sort_order: 10,
        }))
        .to_owned();
    manager.execute(insert).await
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
    for (index, definition) in defaults::api::API_DEFINITIONS.iter().enumerate() {
        insert.values_panic(api_values(index, definition));
    }
    manager.execute(insert.to_owned()).await
}

async fn seed_menu_sections(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(MenuSections::Table).columns([
        MenuSections::Id,
        MenuSections::Code,
        MenuSections::Subheader,
        MenuSections::SortOrder,
        MenuSections::Enabled,
        MenuSections::CreatedAt,
        MenuSections::UpdatedAt,
    ]);
    for definition in defaults::menu::MENU_SECTIONS {
        insert.values_panic(menu_section_values(definition));
    }
    manager.execute(insert.to_owned()).await
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
    for definition in defaults::menu::MENU_ITEMS {
        insert.values_panic(menu_item_values(definition));
    }
    manager.execute(insert.to_owned()).await
}

async fn seed_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_menu_api_bindings(manager).await?;
    seed_role_menu_bindings(manager).await?;
    seed_role_api_bindings(manager).await
}

async fn seed_menu_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(MenuApiPermissions::Table).columns([
        MenuApiPermissions::MenuItemId,
        MenuApiPermissions::ApiPermissionId,
        MenuApiPermissions::CreatedAt,
        MenuApiPermissions::UpdatedAt,
    ]);
    for binding in defaults::MENU_API_BINDINGS {
        for api_code in binding.api_codes {
            insert.values_panic(menu_api_values(binding.menu_code, api_code));
        }
    }
    manager.execute(insert.to_owned()).await
}

async fn seed_role_menu_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(RoleMenuPermissions::Table).columns([
        RoleMenuPermissions::RoleCode,
        RoleMenuPermissions::MenuItemId,
        RoleMenuPermissions::CreatedAt,
        RoleMenuPermissions::UpdatedAt,
    ]);
    for item_id in menu_ids_for_codes(defaults::admin_menu_codes()) {
        insert.values_panic(role_menu_values(defaults::ADMIN_ROLE, item_id));
    }
    for item_id in menu_ids_for_codes(defaults::USER_MENU_CODES) {
        insert.values_panic(role_menu_values(defaults::USER_ROLE, item_id));
    }
    manager.execute(insert.to_owned()).await
}

async fn seed_role_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if defaults::ROLE_API_BINDINGS.is_empty() {
        return Ok(());
    }
    let mut insert = Query::insert();
    insert.into_table(RoleApiPermissions::Table).columns([
        RoleApiPermissions::RoleCode,
        RoleApiPermissions::ApiPermissionId,
        RoleApiPermissions::CreatedAt,
        RoleApiPermissions::UpdatedAt,
    ]);
    for binding in defaults::ROLE_API_BINDINGS {
        for api_code in binding.api_codes {
            insert.values_panic(role_api_values(binding.role_code, api_code));
        }
    }
    manager.execute(insert.to_owned()).await
}

fn role_values(role: RoleSeed) -> [Expr; 8] {
    [
        role.code.into(),
        role.name.into(),
        role.description.into(),
        true.into(),
        true.into(),
        role.sort_order.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

struct RoleSeed {
    code: &'static str,
    name: &'static str,
    description: &'static str,
    sort_order: i64,
}

fn api_values(index: usize, definition: &defaults::api::ApiDefinition) -> [Expr; 10] {
    [
        default_api_id(index).into(),
        definition.code.into(),
        definition.method.into(),
        definition.path_pattern.into(),
        definition.name.into(),
        definition.group.into(),
        true.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn menu_section_values(definition: &defaults::menu::MenuSectionDefinition) -> [Expr; 7] {
    [
        definition.id.into(),
        definition.code.into(),
        definition.subheader.into(),
        definition.sort_order.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn menu_item_values(definition: &defaults::menu::MenuItemDefinition) -> [Expr; 13] {
    [
        definition.id.into(),
        definition.section_id.into(),
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
    ]
}

fn menu_api_values(menu_code: &str, api_code: &str) -> [Expr; 4] {
    [
        menu_id_for_code(menu_code).into(),
        api_id_for_code(api_code).into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn role_menu_values(role_code: &str, item_id: &str) -> [Expr; 4] {
    [role_code.into(), item_id.into(), Expr::current_timestamp(), Expr::current_timestamp()]
}

fn role_api_values(role_code: &str, api_code: &str) -> [Expr; 4] {
    [
        role_code.into(),
        api_id_for_code(api_code).into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn menu_ids_for_codes(codes: impl IntoIterator<Item = impl AsRef<str>>) -> impl Iterator<Item = &'static str> {
    codes.into_iter().map(|code| menu_id_for_code(code.as_ref()))
}

fn menu_id_for_code(code: &str) -> &'static str {
    defaults::menu::MENU_ITEMS
        .iter()
        .find(|item| item.code == code)
        .map(|item| item.id)
        .expect("default menu code must exist")
}

fn api_id_for_code(code: &str) -> String {
    let index = defaults::api::API_DEFINITIONS
        .iter()
        .position(|item| item.code == code)
        .expect("default API code must exist");
    default_api_id(index)
}

fn default_api_id(index: usize) -> String {
    format!("00000000-0000-7000-8000-000000000{:03}", 301 + index)
}
