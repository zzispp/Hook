use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement, TransactionTrait},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const DESTRUCTIVE_VERSION: &str = "m20260807_000001_models_dev_api_removal_destructive";
const MIGRATION_TABLE: &str = "seaql_migrations";
const CLEANUP_SQL: &[&str] = &[
    "DELETE FROM menu_api_permissions WHERE api_permission_id IN (SELECT id FROM api_permissions WHERE code = 'models_external_read')",
    "DELETE FROM role_api_permissions WHERE api_permission_id IN (SELECT id FROM api_permissions WHERE code = 'models_external_read')",
    "DELETE FROM api_permissions WHERE code = 'models_external_read'",
];

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if destructive_marker_exists(manager).await? {
        return Ok(());
    }
    remove_api_metadata(manager).await?;
    mark_destructive_applied(manager).await
}

async fn remove_api_metadata(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let transaction = manager.get_connection().begin().await?;
    for sql in CLEANUP_SQL {
        let statement = Statement::from_string(manager.get_database_backend(), (*sql).to_owned());
        transaction.execute_raw(statement).await?;
    }
    transaction.commit().await
}

async fn destructive_marker_exists(manager: &SchemaManager<'_>) -> Result<bool, DbErr> {
    if !manager.has_table(MIGRATION_TABLE).await? {
        return Ok(false);
    }
    seaql_migrations::Entity::find()
        .filter(seaql_migrations::Column::Version.eq(DESTRUCTIVE_VERSION))
        .one(manager.get_connection())
        .await
        .map(|record| record.is_some())
}

async fn mark_destructive_applied(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_migration_table(manager).await?;
    if destructive_marker_exists(manager).await? {
        return Ok(());
    }
    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(DESTRUCTIVE_VERSION.to_owned()),
        applied_at: ActiveValue::Set(current_timestamp()?),
    })
    .exec(manager.get_connection())
    .await?;
    Ok(())
}

async fn create_migration_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let schema = Schema::new(manager.get_database_backend());
    let mut statement = schema.create_table_from_entity(seaql_migrations::Entity);
    statement.if_not_exists();
    manager.create_table(statement).await
}

fn current_timestamp() -> Result<i64, DbErr> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| DbErr::Migration(format!("system time is before UNIX epoch: {error}")))
}
