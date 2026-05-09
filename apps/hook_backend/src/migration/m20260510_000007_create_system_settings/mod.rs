use sea_orm_migration::{MigrationName, prelude::*, schema::*};

pub struct Migration;

const SETTINGS_ID: &str = "global";
const SETTINGS_MENU_ID: &str = "00000000-0000-7000-8000-000000000213";
const SYSTEM_SECTION_ID: &str = "00000000-0000-7000-8000-000000000103";
const ADMIN_ROLE: &str = "admin";
const API_BASE_ID: usize = 431;

#[derive(DeriveIden)]
enum SystemSettings {
    Table,
    Id,
    SiteName,
    SiteSubtitle,
    AllowRegistration,
    AutoDeleteExpiredTokens,
    DefaultUserGrant,
    DefaultRateLimitRpm,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiPermissions {
    Table,
    Id,
    Code,
    Method,
    PathPattern,
    Name,
    Group,
    Enabled,
    System,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
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

#[derive(DeriveIden)]
enum MenuApiPermissions {
    Table,
    MenuItemId,
    ApiPermissionId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RoleMenuPermissions {
    Table,
    RoleCode,
    MenuItemId,
    CreatedAt,
    UpdatedAt,
}

struct ApiDefinition {
    code: &'static str,
    method: &'static str,
    path_pattern: &'static str,
    name: &'static str,
    group: &'static str,
}

const API_DEFINITIONS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "system_settings_read",
        method: "GET",
        path_pattern: "/api/admin/settings/system",
        name: "Read system settings",
        group: "System Settings",
    },
    ApiDefinition {
        code: "system_settings_update",
        method: "PATCH",
        path_pattern: "/api/admin/settings/system",
        name: "Update system settings",
        group: "System Settings",
    },
];

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260510_000007_create_system_settings"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(settings_table()).await?;
        seed_settings(manager).await?;
        seed_api_permissions(manager).await?;
        seed_menu_item(manager).await?;
        seed_menu_api_bindings(manager).await?;
        seed_role_menu_binding(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        delete_role_menu_binding(manager).await?;
        delete_menu_api_bindings(manager).await?;
        delete_menu_item(manager).await?;
        delete_api_permissions(manager).await?;
        manager.drop_table(Table::drop().table(SystemSettings::Table).if_exists().to_owned()).await
    }
}

fn settings_table() -> TableCreateStatement {
    Table::create()
        .table(SystemSettings::Table)
        .if_not_exists()
        .col(string_len(SystemSettings::Id, 36).primary_key())
        .col(string_len(SystemSettings::SiteName, 100))
        .col(string_len(SystemSettings::SiteSubtitle, 200))
        .col(boolean(SystemSettings::AllowRegistration))
        .col(boolean(SystemSettings::AutoDeleteExpiredTokens))
        .col(decimal_len(SystemSettings::DefaultUserGrant, 20, 8))
        .col(big_integer(SystemSettings::DefaultRateLimitRpm))
        .col(timestamp_tz(SystemSettings::CreatedAt))
        .col(timestamp_tz(SystemSettings::UpdatedAt))
        .to_owned()
}

async fn seed_settings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(SystemSettings::Table)
                .columns([
                    SystemSettings::Id,
                    SystemSettings::SiteName,
                    SystemSettings::SiteSubtitle,
                    SystemSettings::AllowRegistration,
                    SystemSettings::AutoDeleteExpiredTokens,
                    SystemSettings::DefaultUserGrant,
                    SystemSettings::DefaultRateLimitRpm,
                    SystemSettings::CreatedAt,
                    SystemSettings::UpdatedAt,
                ])
                .values_panic([
                    SETTINGS_ID.into(),
                    "Hook".into(),
                    "AI API platform".into(),
                    true.into(),
                    false.into(),
                    0.into(),
                    0.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .on_conflict(OnConflict::column(SystemSettings::Id).do_nothing().to_owned())
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

async fn seed_menu_item(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(MenuItems::Table)
                .columns([
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
                ])
                .values_panic([
                    SETTINGS_MENU_ID.into(),
                    SYSTEM_SECTION_ID.into(),
                    Option::<String>::None.into(),
                    "admin_settings".into(),
                    "System Settings".into(),
                    "/dashboard/admin/settings".into(),
                    Some("icon.settings".to_owned()).into(),
                    Option::<String>::None.into(),
                    true.into(),
                    65.into(),
                    true.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
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
    for code in API_DEFINITIONS.iter().map(|api| api.code) {
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
                .values_panic([ADMIN_ROLE.into(), SETTINGS_MENU_ID.into(), Expr::current_timestamp(), Expr::current_timestamp()])
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
                .and_where(Expr::col(RoleMenuPermissions::MenuItemId).eq(SETTINGS_MENU_ID))
                .to_owned(),
        )
        .await
}

async fn delete_menu_api_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(MenuApiPermissions::Table)
                .and_where(Expr::col(MenuApiPermissions::MenuItemId).eq(SETTINGS_MENU_ID))
                .to_owned(),
        )
        .await
}

async fn delete_menu_item(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(MenuItems::Table)
                .and_where(Expr::col(MenuItems::Code).eq("admin_settings"))
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
        SETTINGS_MENU_ID.into(),
        api_id_for_code(code)?.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ])
}

fn api_id_for_code(code: &str) -> Result<String, DbErr> {
    API_DEFINITIONS
        .iter()
        .position(|api| api.code == code)
        .map(default_api_id)
        .ok_or_else(|| DbErr::Custom(format!("seed api code does not exist: {code}")))
}

fn default_api_id(index: usize) -> String {
    format!("00000000-0000-7000-8000-000000000{:03}", API_BASE_ID + index)
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}
