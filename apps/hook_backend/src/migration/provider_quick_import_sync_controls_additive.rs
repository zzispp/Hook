use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260610_000003_provider_quick_import_sync_controls";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    execute_sql_batch(manager, COLUMN_SQL).await?;
    execute_sql_batch(manager, TABLE_SQL).await?;
    execute_sql_batch(manager, INDEX_SQL).await?;
    mark_additive_applied(manager).await
}

async fn execute_sql_batch(manager: &SchemaManager<'_>, sql_items: &[&str]) -> Result<(), DbErr> {
    for sql in sql_items {
        manager
            .get_connection()
            .execute_raw(Statement::from_string(manager.get_database_backend(), (*sql).to_owned()))
            .await?;
    }
    Ok(())
}

async fn additive_marker_exists(manager: &SchemaManager<'_>) -> Result<bool, DbErr> {
    if !manager.has_table(MIGRATION_TABLE).await? {
        return Ok(false);
    }
    seaql_migrations::Entity::find()
        .filter(seaql_migrations::Column::Version.eq(ADDITIVE_VERSION))
        .one(manager.get_connection())
        .await
        .map(|record| record.is_some())
}

async fn mark_additive_applied(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_migration_table(manager).await?;
    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(ADDITIVE_VERSION.to_owned()),
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

const COLUMN_SQL: &[&str] = &[
    "ALTER TABLE provider_quick_import_sources ADD COLUMN IF NOT EXISTS token_deleted_action VARCHAR(32)",
    "ALTER TABLE provider_quick_import_sources ADD COLUMN IF NOT EXISTS token_disabled_action VARCHAR(32)",
    "ALTER TABLE provider_quick_import_sources ADD COLUMN IF NOT EXISTS group_removed_action VARCHAR(32)",
    "ALTER TABLE provider_quick_import_sources ADD COLUMN IF NOT EXISTS group_changed_action VARCHAR(32)",
    "ALTER TABLE provider_quick_import_sources ADD COLUMN IF NOT EXISTS key_unavailable_action VARCHAR(32)",
    "ALTER TABLE provider_quick_import_sources ADD COLUMN IF NOT EXISTS model_removed_action VARCHAR(32)",
    "UPDATE provider_quick_import_sources SET token_deleted_action = upstream_anomaly_action WHERE token_deleted_action IS NULL",
    "UPDATE provider_quick_import_sources SET token_disabled_action = upstream_anomaly_action WHERE token_disabled_action IS NULL",
    "UPDATE provider_quick_import_sources SET group_removed_action = upstream_anomaly_action WHERE group_removed_action IS NULL",
    "UPDATE provider_quick_import_sources SET group_changed_action = upstream_anomaly_action WHERE group_changed_action IS NULL",
    "UPDATE provider_quick_import_sources SET key_unavailable_action = upstream_anomaly_action WHERE key_unavailable_action IS NULL",
    "UPDATE provider_quick_import_sources SET model_removed_action = upstream_anomaly_action WHERE model_removed_action IS NULL",
    "ALTER TABLE provider_quick_import_sources ALTER COLUMN token_deleted_action SET NOT NULL",
    "ALTER TABLE provider_quick_import_sources ALTER COLUMN token_disabled_action SET NOT NULL",
    "ALTER TABLE provider_quick_import_sources ALTER COLUMN group_removed_action SET NOT NULL",
    "ALTER TABLE provider_quick_import_sources ALTER COLUMN group_changed_action SET NOT NULL",
    "ALTER TABLE provider_quick_import_sources ALTER COLUMN key_unavailable_action SET NOT NULL",
    "ALTER TABLE provider_quick_import_sources ALTER COLUMN model_removed_action SET NOT NULL",
];

const TABLE_SQL: &[&str] = &["CREATE TABLE IF NOT EXISTS provider_quick_import_sync_events (\
        id VARCHAR(36) PRIMARY KEY,\
        provider_id VARCHAR(36) NOT NULL REFERENCES providers(id) ON DELETE CASCADE,\
        source_id VARCHAR(36) NOT NULL REFERENCES provider_quick_import_sources(id) ON DELETE CASCADE,\
        key_id VARCHAR(36) REFERENCES provider_api_keys(id) ON DELETE SET NULL,\
        status VARCHAR(64) NOT NULL,\
        title TEXT NOT NULL,\
        detail TEXT NOT NULL,\
        created_at TIMESTAMPTZ NOT NULL\
    )"];

const INDEX_SQL: &[&str] = &["CREATE INDEX IF NOT EXISTS index_provider_quick_import_sync_events_by_created ON provider_quick_import_sync_events(created_at)"];
