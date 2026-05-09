use sea_orm_migration::{MigrationName, prelude::*, schema::*};

pub struct Migration;

const ADMIN_GROUP_MENU_ID: &str = "00000000-0000-7000-8000-000000000211";

#[derive(DeriveIden)]
enum BillingGroupModels {
    Table,
    Id,
    GroupCode,
    GlobalModelId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BillingGroups {
    Table,
    Code,
}

#[derive(DeriveIden)]
enum GlobalModels {
    Table,
    Id,
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
        "m20260510_000006_create_billing_group_model_bindings"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(binding_table()).await?;
        manager.create_index(binding_unique_index()).await?;
        manager.create_index(group_index()).await?;
        bind_model_read_to_admin_groups(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        unbind_model_read_from_admin_groups(manager).await?;
        manager.drop_table(Table::drop().table(BillingGroupModels::Table).if_exists().to_owned()).await
    }
}

fn binding_table() -> TableCreateStatement {
    let mut group_foreign_key = group_fk();
    let mut model_foreign_key = global_model_fk();
    Table::create()
        .table(BillingGroupModels::Table)
        .if_not_exists()
        .col(string_len(BillingGroupModels::Id, 36).primary_key())
        .col(string_len(BillingGroupModels::GroupCode, 64))
        .col(string_len(BillingGroupModels::GlobalModelId, 36))
        .col(timestamp_tz(BillingGroupModels::CreatedAt))
        .col(timestamp_tz(BillingGroupModels::UpdatedAt))
        .foreign_key(&mut group_foreign_key)
        .foreign_key(&mut model_foreign_key)
        .to_owned()
}

fn binding_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_billing_group_models_unique")
        .table(BillingGroupModels::Table)
        .col(BillingGroupModels::GroupCode)
        .col(BillingGroupModels::GlobalModelId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn group_index() -> IndexCreateStatement {
    Index::create()
        .name("index_billing_group_models_by_group")
        .table(BillingGroupModels::Table)
        .col(BillingGroupModels::GroupCode)
        .if_not_exists()
        .to_owned()
}

fn group_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_models_group")
        .from(BillingGroupModels::Table, BillingGroupModels::GroupCode)
        .to(BillingGroups::Table, BillingGroups::Code)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn global_model_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_billing_group_models_global_model")
        .from(BillingGroupModels::Table, BillingGroupModels::GlobalModelId)
        .to(GlobalModels::Table, GlobalModels::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

async fn bind_model_read_to_admin_groups(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let api_id = select_api_id(manager, "models_global_read").await?;
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
                .values_panic([ADMIN_GROUP_MENU_ID.into(), api_id.into(), Expr::current_timestamp(), Expr::current_timestamp()])
                .on_conflict(
                    OnConflict::columns([MenuApiPermissions::MenuItemId, MenuApiPermissions::ApiPermissionId])
                        .do_nothing()
                        .to_owned(),
                )
                .to_owned(),
        )
        .await
}

async fn unbind_model_read_from_admin_groups(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let api_id = select_api_id(manager, "models_global_read").await?;
    manager
        .execute(
            Query::delete()
                .from_table(MenuApiPermissions::Table)
                .and_where(Expr::col(MenuApiPermissions::MenuItemId).eq(ADMIN_GROUP_MENU_ID))
                .and_where(Expr::col(MenuApiPermissions::ApiPermissionId).eq(api_id))
                .to_owned(),
        )
        .await
}

async fn select_api_id(manager: &SchemaManager<'_>, code: &str) -> Result<String, DbErr> {
    let statement = Query::select()
        .column(ApiPermissions::Id)
        .from(ApiPermissions::Table)
        .and_where(Expr::col(ApiPermissions::Code).eq(code))
        .to_owned();
    let row = manager.get_connection().query_one(&statement).await?;
    let row = row.ok_or_else(|| DbErr::Custom(format!("{code} api permission is missing")))?;
    row.try_get("", "id")
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}
