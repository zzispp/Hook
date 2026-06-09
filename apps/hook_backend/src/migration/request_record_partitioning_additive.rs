use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260609_000004_request_record_partitioning";
const MIGRATION_TABLE: &str = "seaql_migrations";
const INITIAL_PARTITION_FUTURE_DAYS: i64 = 3;

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    create_partitioned_tables(manager).await?;
    create_partitioned_indexes(manager).await?;
    create_initial_partitions(manager).await?;
    mark_additive_applied(manager).await
}

async fn create_partitioned_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(
        manager,
        "CREATE TABLE IF NOT EXISTS request_records_partitioned (LIKE request_records INCLUDING DEFAULTS INCLUDING STORAGE) PARTITION BY RANGE (created_at)",
    )
    .await?;
    execute_sql(manager, "ALTER TABLE request_records_partitioned DROP COLUMN IF EXISTS request_headers, DROP COLUMN IF EXISTS request_body, DROP COLUMN IF EXISTS client_response_headers, DROP COLUMN IF EXISTS client_response_body, DROP COLUMN IF EXISTS payload_compressed_at").await?;
    execute_sql(manager, add_primary_key_sql("request_records_partitioned", "created_at, request_id")).await?;
    execute_sql(manager, "CREATE TABLE IF NOT EXISTS request_candidates_partitioned (LIKE request_candidates INCLUDING DEFAULTS INCLUDING STORAGE) PARTITION BY RANGE (created_at)").await?;
    execute_sql(manager, "ALTER TABLE request_candidates_partitioned DROP COLUMN IF EXISTS provider_request_headers, DROP COLUMN IF EXISTS provider_request_body, DROP COLUMN IF EXISTS provider_response_headers, DROP COLUMN IF EXISTS provider_response_body, DROP COLUMN IF EXISTS payload_compressed_at").await?;
    execute_sql(manager, add_primary_key_sql("request_candidates_partitioned", "created_at, id")).await?;
    execute_sql(manager, request_payloads_table_sql()).await?;
    Ok(())
}

async fn create_partitioned_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in partitioned_index_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn create_initial_partitions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let today = time::OffsetDateTime::now_utc().date();
    for offset in -1..=INITIAL_PARTITION_FUTURE_DAYS {
        let day = today + time::Duration::days(offset);
        create_day_partitions(manager, day).await?;
    }
    Ok(())
}

async fn create_day_partitions(manager: &SchemaManager<'_>, day: time::Date) -> Result<(), DbErr> {
    for (parent, prefix) in partition_families() {
        execute_sql(manager, &create_partition_sql(parent, prefix, day)).await?;
    }
    Ok(())
}

fn add_primary_key_sql(table: &str, columns: &str) -> String {
    format!(
        "DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = '{table}_pkey' AND conrelid = '{table}'::regclass) THEN ALTER TABLE {table} ADD CONSTRAINT {table}_pkey PRIMARY KEY ({columns}); END IF; END $$"
    )
}

fn request_payloads_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS request_payloads (\
     created_at TIMESTAMPTZ NOT NULL, \
     owner_type VARCHAR(32) NOT NULL, \
     owner_id VARCHAR(64) NOT NULL, \
     payload_kind VARCHAR(64) NOT NULL, \
     status VARCHAR(32) NOT NULL, \
     source VARCHAR(32) NOT NULL, \
     original_size BIGINT NULL, \
     compressed_size BIGINT NULL, \
     sha256 VARCHAR(64) NULL, \
     compressed_payload BYTEA NULL, \
     error_message TEXT NULL, \
     updated_at TIMESTAMPTZ NOT NULL, \
     PRIMARY KEY (created_at, owner_type, owner_id, payload_kind)) PARTITION BY RANGE (created_at)"
}

fn partitioned_index_sql() -> [&'static str; 17] {
    [
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_request ON request_records_partitioned (request_id)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_created ON request_records_partitioned (created_at DESC, request_id DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_user_created ON request_records_partitioned (user_id_snapshot, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_token_created ON request_records_partitioned (token_id, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_status_created ON request_records_partitioned (status, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_model_created ON request_records_partitioned (global_model_id, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_provider_created ON request_records_partitioned (provider_id, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_client_format ON request_records_partitioned (client_api_format, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_provider_format ON request_records_partitioned (provider_api_format, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_stream_created ON request_records_partitioned (is_stream, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_records_partitioned_by_error_created ON request_records_partitioned (client_error_type, created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_candidates_partitioned_by_request ON request_candidates_partitioned (request_id)",
        "CREATE INDEX IF NOT EXISTS index_request_candidates_partitioned_by_attempt ON request_candidates_partitioned (request_id, candidate_index, retry_index)",
        "CREATE INDEX IF NOT EXISTS index_request_candidates_partitioned_by_created ON request_candidates_partitioned (created_at DESC, id DESC)",
        "CREATE INDEX IF NOT EXISTS index_request_payloads_by_owner ON request_payloads (owner_type, owner_id, payload_kind)",
        "CREATE INDEX IF NOT EXISTS index_request_payloads_by_status_updated ON request_payloads (status, updated_at)",
        "CREATE INDEX IF NOT EXISTS index_request_payloads_by_created ON request_payloads (created_at DESC)",
    ]
}

fn partition_families() -> [(&'static str, &'static str); 3] {
    [
        ("request_records_partitioned", "request_records_partitioned"),
        ("request_candidates_partitioned", "request_candidates_partitioned"),
        ("request_payloads", "request_payloads"),
    ]
}

fn create_partition_sql(parent: &str, prefix: &str, day: time::Date) -> String {
    let name = format!("{}_{}", prefix, compact_date(day));
    let next_day = day + time::Duration::days(1);
    format!(
        "CREATE TABLE IF NOT EXISTS \"{name}\" PARTITION OF \"{parent}\" FOR VALUES FROM ('{}') TO ('{}')",
        iso_date(day),
        iso_date(next_day)
    )
}

fn compact_date(day: time::Date) -> String {
    format!("{:04}{:02}{:02}", day.year(), u8::from(day.month()), day.day())
}

fn iso_date(day: time::Date) -> String {
    format!("{:04}-{:02}-{:02}", day.year(), u8::from(day.month()), day.day())
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

#[cfg(test)]
mod tests {
    use super::{create_partition_sql, request_payloads_table_sql};

    #[test]
    fn payloads_table_is_partitioned_by_created_at() {
        assert!(request_payloads_table_sql().contains("PARTITION BY RANGE (created_at)"));
        assert!(request_payloads_table_sql().contains("PRIMARY KEY (created_at, owner_type, owner_id, payload_kind)"));
    }

    #[test]
    fn partition_names_use_daily_suffix() {
        let day = time::Date::from_calendar_date(2026, time::Month::June, 9).unwrap();

        assert_eq!(
            create_partition_sql("request_payloads", "request_payloads", day),
            "CREATE TABLE IF NOT EXISTS \"request_payloads_20260609\" PARTITION OF \"request_payloads\" FOR VALUES FROM ('2026-06-09') TO ('2026-06-10')"
        );
    }
}
