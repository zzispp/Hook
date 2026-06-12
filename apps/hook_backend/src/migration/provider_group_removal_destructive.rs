use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const DESTRUCTIVE_VERSION: &str = "m20260611_000001_remove_provider_groups_destructive";
const MIGRATION_TABLE: &str = "seaql_migrations";
const DROP_TABLE_SQL: &[&str] = &[
    "DROP TABLE IF EXISTS billing_group_provider_groups",
    "DROP TABLE IF EXISTS provider_group_providers",
    "DROP TABLE IF EXISTS provider_groups",
];

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if destructive_marker_exists(manager).await? {
        return Ok(());
    }
    drop_legacy_tables(manager).await?;
    mark_destructive_applied(manager).await
}

pub async fn drop_legacy_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in DROP_TABLE_SQL {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_raw(Statement::from_string(manager.get_database_backend(), sql.to_owned()))
        .await
        .map(|_| ())
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
