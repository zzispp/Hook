use sea_orm_migration::{MigrationName, prelude::*};

pub struct Migration;

#[derive(DeriveIden)]
enum BillingGroups {
    Table,
    Code,
    Name,
    Description,
    IsActive,
    IsSystem,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiPermissions {
    Table,
    Id,
    Code,
}

#[derive(DeriveIden)]
enum MenuApiPermissions {
    Table,
    MenuItemId,
    ApiPermissionId,
    CreatedAt,
    UpdatedAt,
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260510_000005_system_group_and_model_group_pricing"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        update_system_group(manager).await?;
        bind_model_catalog_to_groups(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        unbind_model_catalog_from_groups(manager).await
    }
}

async fn update_system_group(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::update()
                .table(BillingGroups::Table)
                .values([
                    (BillingGroups::Name, "System Group".into()),
                    (
                        BillingGroups::Description,
                        Some("Built-in billing group used when a token does not choose a group").into(),
                    ),
                    (BillingGroups::IsActive, true.into()),
                    (BillingGroups::IsSystem, true.into()),
                    (BillingGroups::UpdatedAt, Expr::current_timestamp()),
                ])
                .and_where(Expr::col(BillingGroups::Code).eq(constants::billing::DEFAULT_SYSTEM_GROUP_CODE))
                .to_owned(),
        )
        .await
}

async fn bind_model_catalog_to_groups(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let api_id = select_groups_available_api_id(manager).await?;
    manager
        .execute(
            Query::insert()
                .into_table(MenuApiPermissions::Table)
                .columns([
                    MenuApiPermissions::MenuItemId,
                    MenuApiPermissions::ApiPermissionId,
                    MenuApiPermissions::CreatedAt,
                    MenuApiPermissions::UpdatedAt,
                ])
                .values_panic([
                    model_catalog_menu_id().into(),
                    api_id.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .on_conflict(
                    OnConflict::columns([MenuApiPermissions::MenuItemId, MenuApiPermissions::ApiPermissionId])
                        .do_nothing()
                        .to_owned(),
                )
                .to_owned(),
        )
        .await
}

async fn unbind_model_catalog_from_groups(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let api_id = select_groups_available_api_id(manager).await?;
    manager
        .execute(
            Query::delete()
                .from_table(MenuApiPermissions::Table)
                .and_where(Expr::col(MenuApiPermissions::MenuItemId).eq(model_catalog_menu_id()))
                .and_where(Expr::col(MenuApiPermissions::ApiPermissionId).eq(api_id))
                .to_owned(),
        )
        .await
}

async fn select_groups_available_api_id(manager: &SchemaManager<'_>) -> Result<String, DbErr> {
    let statement = Query::select()
        .column(ApiPermissions::Id)
        .from(ApiPermissions::Table)
        .and_where(Expr::col(ApiPermissions::Code).eq("groups_available_read"))
        .to_owned();
    let row = manager.get_connection().query_one(&statement).await?;
    let row = row.ok_or_else(|| DbErr::Custom("groups_available_read api permission is missing".into()))?;
    row.try_get("", "id")
}

fn model_catalog_menu_id() -> &'static str {
    "00000000-0000-7000-8000-000000000202"
}
