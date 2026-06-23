use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260623_000001_stream_tail_latency";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    for sql in request_record_sql() {
        execute_sql(manager, sql).await?;
    }
    for sql in request_candidate_sql() {
        execute_sql(manager, sql).await?;
    }
    for sql in routing_metric_sql() {
        execute_sql(manager, sql).await?;
    }
    mark_additive_applied(manager).await
}

fn request_record_sql() -> [&'static str; 6] {
    [
        "ALTER TABLE IF EXISTS request_records ADD COLUMN IF NOT EXISTS response_headers_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_records ADD COLUMN IF NOT EXISTS first_sse_event_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_records ADD COLUMN IF NOT EXISTS first_output_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_records_partitioned ADD COLUMN IF NOT EXISTS response_headers_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_records_partitioned ADD COLUMN IF NOT EXISTS first_sse_event_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_records_partitioned ADD COLUMN IF NOT EXISTS first_output_time_ms BIGINT NULL",
    ]
}

fn request_candidate_sql() -> [&'static str; 6] {
    [
        "ALTER TABLE IF EXISTS request_candidates ADD COLUMN IF NOT EXISTS response_headers_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_candidates ADD COLUMN IF NOT EXISTS first_sse_event_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_candidates ADD COLUMN IF NOT EXISTS first_output_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_candidates_partitioned ADD COLUMN IF NOT EXISTS response_headers_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_candidates_partitioned ADD COLUMN IF NOT EXISTS first_sse_event_time_ms BIGINT NULL",
        "ALTER TABLE IF EXISTS request_candidates_partitioned ADD COLUMN IF NOT EXISTS first_output_time_ms BIGINT NULL",
    ]
}

fn routing_metric_sql() -> [&'static str; 2] {
    [
        "ALTER TABLE IF EXISTS routing_metric_buckets ADD COLUMN IF NOT EXISTS first_output_success_count BIGINT NOT NULL DEFAULT 0",
        "ALTER TABLE IF EXISTS routing_metric_buckets ADD COLUMN IF NOT EXISTS first_output_failure_count BIGINT NOT NULL DEFAULT 0",
    ]
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: impl Into<String>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_raw(Statement::from_string(manager.get_database_backend(), sql.into()))
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
