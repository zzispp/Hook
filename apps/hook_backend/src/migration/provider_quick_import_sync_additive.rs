use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260610_000002_provider_quick_import_sync";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    create_tables(manager).await?;
    create_indexes(manager).await?;
    mark_additive_applied(manager).await
}

async fn create_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in TABLE_SQL {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in INDEX_SQL {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_raw(Statement::from_string(manager.get_database_backend(), sql.to_owned()))
        .await?;
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
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
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

const TABLE_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS provider_quick_import_sources (\
        id VARCHAR(36) PRIMARY KEY,\
        provider_id VARCHAR(36) NOT NULL REFERENCES providers(id) ON DELETE CASCADE,\
        source_kind VARCHAR(32) NOT NULL,\
        base_url TEXT NOT NULL,\
        encrypted_system_access_token TEXT NOT NULL,\
        user_id VARCHAR(100) NOT NULL,\
        recharge_multiplier DECIMAL(20,8) NOT NULL,\
        auto_sync_enabled BOOLEAN NOT NULL,\
        cost_sync_mode VARCHAR(32) NOT NULL,\
        upstream_anomaly_action VARCHAR(32) NOT NULL,\
        fetch_failure_action VARCHAR(32) NOT NULL,\
        fetch_failure_disable_threshold INTEGER NOT NULL,\
        last_status VARCHAR(64),\
        last_error TEXT,\
        last_synced_at TIMESTAMPTZ,\
        consecutive_failures INTEGER NOT NULL DEFAULT 0,\
        created_at TIMESTAMPTZ NOT NULL,\
        updated_at TIMESTAMPTZ NOT NULL\
    )",
    "CREATE TABLE IF NOT EXISTS provider_quick_import_keys (\
        id VARCHAR(36) PRIMARY KEY,\
        provider_id VARCHAR(36) NOT NULL REFERENCES providers(id) ON DELETE CASCADE,\
        source_id VARCHAR(36) NOT NULL REFERENCES provider_quick_import_sources(id) ON DELETE CASCADE,\
        key_id VARCHAR(36) NOT NULL REFERENCES provider_api_keys(id) ON DELETE CASCADE,\
        upstream_token_id VARCHAR(100) NOT NULL,\
        upstream_token_name VARCHAR(100) NOT NULL,\
        upstream_masked_key VARCHAR(200) NOT NULL,\
        upstream_group VARCHAR(100),\
        upstream_group_ratio DECIMAL(20,8) NOT NULL,\
        effective_cost_multiplier DECIMAL(20,8) NOT NULL,\
        sync_statuses TEXT NOT NULL,\
        last_sync_error TEXT,\
        last_synced_at TIMESTAMPTZ,\
        created_at TIMESTAMPTZ NOT NULL,\
        updated_at TIMESTAMPTZ NOT NULL\
    )",
    "CREATE TABLE IF NOT EXISTS provider_quick_import_key_models (\
        id VARCHAR(36) PRIMARY KEY,\
        provider_id VARCHAR(36) NOT NULL REFERENCES providers(id) ON DELETE CASCADE,\
        source_id VARCHAR(36) NOT NULL REFERENCES provider_quick_import_sources(id) ON DELETE CASCADE,\
        key_id VARCHAR(36) NOT NULL REFERENCES provider_api_keys(id) ON DELETE CASCADE,\
        upstream_model_id VARCHAR(200) NOT NULL,\
        global_model_id VARCHAR(36) NOT NULL REFERENCES global_models(id) ON DELETE CASCADE,\
        created_at TIMESTAMPTZ NOT NULL,\
        updated_at TIMESTAMPTZ NOT NULL\
    )",
];

const INDEX_SQL: &[&str] = &[
    "CREATE UNIQUE INDEX IF NOT EXISTS index_provider_quick_import_sources_provider_unique ON provider_quick_import_sources(provider_id)",
    "CREATE UNIQUE INDEX IF NOT EXISTS index_provider_quick_import_keys_key_unique ON provider_quick_import_keys(key_id)",
    "CREATE INDEX IF NOT EXISTS index_provider_quick_import_keys_by_source ON provider_quick_import_keys(source_id)",
    "CREATE UNIQUE INDEX IF NOT EXISTS index_provider_quick_import_key_models_unique ON provider_quick_import_key_models(key_id, upstream_model_id)",
    "CREATE INDEX IF NOT EXISTS index_provider_quick_import_key_models_by_key ON provider_quick_import_key_models(key_id)",
];
