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
        .columns([Roles::Code, Roles::Name, Roles::Description, Roles::Enabled, Roles::System, Roles::SortOrder])
        .values_panic(role_values(defaults::ADMIN_ROLE, "Administrator", "Built-in administrator role", 0))
        .values_panic(role_values(defaults::USER_ROLE, "User", "Default signed-up user role", 10))
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
    ]);
    for definition in defaults::menu::MENU_ITEMS {
        insert.values_panic(menu_item_values(definition));
    }
    manager.execute(insert.to_owned()).await
}

async fn seed_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_role_api_bindings(manager).await?;
    seed_role_menu_bindings(manager).await
}

async fn seed_role_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert
        .into_table(RoleApiPermissions::Table)
        .columns([RoleApiPermissions::RoleCode, RoleApiPermissions::ApiPermissionId]);
    for code in defaults::admin_api_codes() {
        insert.values_panic(role_api_values(defaults::ADMIN_ROLE, code));
    }
    for code in defaults::USER_API_CODES {
        insert.values_panic(role_api_values(defaults::USER_ROLE, code));
    }
    manager.execute(insert.to_owned()).await
}

async fn seed_role_menu_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert
        .into_table(RoleMenuPermissions::Table)
        .columns([RoleMenuPermissions::RoleCode, RoleMenuPermissions::MenuItemId]);
    for item_id in menu_ids_for_codes(defaults::admin_menu_codes()) {
        insert.values_panic(role_menu_values(defaults::ADMIN_ROLE, item_id));
    }
    for item_id in menu_ids_for_codes(defaults::USER_MENU_CODES) {
        insert.values_panic(role_menu_values(defaults::USER_ROLE, item_id));
    }
    manager.execute(insert.to_owned()).await
}

fn role_values(code: &str, name: &str, description: &str, sort_order: i64) -> [Expr; 6] {
    [code.into(), name.into(), description.into(), true.into(), true.into(), sort_order.into()]
}

fn api_values(index: usize, definition: &defaults::api::ApiDefinition) -> [Expr; 8] {
    [
        default_api_id(index).into(),
        definition.code.into(),
        definition.method.into(),
        definition.path_pattern.into(),
        definition.name.into(),
        definition.group.into(),
        true.into(),
        true.into(),
    ]
}

fn menu_section_values(definition: &defaults::menu::MenuSectionDefinition) -> [Expr; 5] {
    [
        definition.id.into(),
        definition.code.into(),
        definition.subheader.into(),
        definition.sort_order.into(),
        true.into(),
    ]
}

fn menu_item_values(definition: &defaults::menu::MenuItemDefinition) -> [Expr; 11] {
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
    ]
}

fn role_api_values(role_code: &str, code: &str) -> [Expr; 2] {
    [role_code.into(), api_id_for_code(code).into()]
}

fn role_menu_values(role_code: &str, item_id: &str) -> [Expr; 2] {
    [role_code.into(), item_id.into()]
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
