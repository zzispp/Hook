use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260609_000002_request_record_payload_compression";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    add_payload_columns(manager).await?;
    create_payload_indexes(manager).await?;
    mark_additive_applied(manager).await
}

async fn add_payload_columns(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(
        manager,
        "ALTER TABLE request_records ADD COLUMN IF NOT EXISTS payload_compressed_at TIMESTAMPTZ NULL",
    )
    .await?;
    execute_sql(
        manager,
        "ALTER TABLE request_candidates ADD COLUMN IF NOT EXISTS payload_compressed_at TIMESTAMPTZ NULL",
    )
    .await
}

async fn create_payload_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(
        manager,
        "CREATE INDEX CONCURRENTLY IF NOT EXISTS index_request_records_payload_cleanup ON request_records (created_at ASC, request_id ASC) \
         WHERE payload_compressed_at IS NULL AND (request_headers IS NOT NULL OR request_body IS NOT NULL OR client_response_headers IS NOT NULL OR client_response_body IS NOT NULL)",
    )
    .await?;
    execute_sql(
        manager,
        "CREATE INDEX CONCURRENTLY IF NOT EXISTS index_request_candidates_payload_cleanup ON request_candidates (created_at ASC, id ASC) \
         WHERE payload_compressed_at IS NULL AND (provider_request_headers IS NOT NULL OR provider_request_body IS NOT NULL OR provider_response_headers IS NOT NULL OR provider_response_body IS NOT NULL)",
    )
    .await
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
