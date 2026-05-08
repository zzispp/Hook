use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in timestamped_tables() {
            add_timestamp_columns(manager, table).await?;
            backfill_timestamp_columns(manager, table.name()).await?;
            require_timestamp_columns(manager, table).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in timestamped_tables().into_iter().rev() {
            drop_timestamp_columns(manager, table).await?;
        }
        Ok(())
    }
}

fn timestamped_tables() -> Vec<AuditTable> {
    vec![
        AuditTable::Roles,
        AuditTable::ApiPermissions,
        AuditTable::MenuSections,
        AuditTable::MenuItems,
        AuditTable::RoleApiPermissions,
        AuditTable::RoleMenuPermissions,
    ]
}

async fn add_timestamp_columns(manager: &SchemaManager<'_>, table: AuditTable) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(table)
                .add_column_if_not_exists(timestamp_tz_null(AuditColumns::CreatedAt))
                .add_column_if_not_exists(timestamp_tz_null(AuditColumns::UpdatedAt))
                .to_owned(),
        )
        .await
}

async fn backfill_timestamp_columns(manager: &SchemaManager<'_>, table_name: &str) -> Result<(), DbErr> {
    let sql = format!(r#"UPDATE "{table_name}" SET "created_at" = CURRENT_TIMESTAMP, "updated_at" = CURRENT_TIMESTAMP"#);
    manager.get_connection().execute_unprepared(&sql).await.map(|_| ())
}

async fn require_timestamp_columns(manager: &SchemaManager<'_>, table: AuditTable) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(table)
                .modify_column(timestamp_tz(AuditColumns::CreatedAt))
                .modify_column(timestamp_tz(AuditColumns::UpdatedAt))
                .to_owned(),
        )
        .await
}

async fn drop_timestamp_columns(manager: &SchemaManager<'_>, table: AuditTable) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(table)
                .drop_column(AuditColumns::UpdatedAt)
                .drop_column(AuditColumns::CreatedAt)
                .to_owned(),
        )
        .await
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

#[derive(Clone, Copy, DeriveIden)]
enum AuditTable {
    Roles,
    ApiPermissions,
    MenuSections,
    MenuItems,
    RoleApiPermissions,
    RoleMenuPermissions,
}

impl AuditTable {
    fn name(self) -> &'static str {
        match self {
            AuditTable::Roles => "roles",
            AuditTable::ApiPermissions => "api_permissions",
            AuditTable::MenuSections => "menu_sections",
            AuditTable::MenuItems => "menu_items",
            AuditTable::RoleApiPermissions => "role_api_permissions",
            AuditTable::RoleMenuPermissions => "role_menu_permissions",
        }
    }
}

#[derive(DeriveIden)]
enum AuditColumns {
    CreatedAt,
    UpdatedAt,
}
