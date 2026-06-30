use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const DESTRUCTIVE_VERSION: &str = "m20260630_000003_stream_timing_first_byte_semantics";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if destructive_marker_exists(manager).await? {
        return Ok(());
    }
    for sql in column_semantics_sql() {
        execute_sql(manager, sql).await?;
    }
    for sql in snapshot_json_sql() {
        execute_sql(manager, sql).await?;
    }
    mark_destructive_applied(manager).await
}

fn column_semantics_sql() -> Vec<String> {
    let mut sql = Vec::new();
    sql.extend(counter_sql("dashboard_request_metric_buckets", "first_byte_total_ms", "ttfb_total_ms"));
    sql.extend(counter_sql("dashboard_request_metric_buckets", "first_byte_sample_count", "ttfb_sample_count"));
    sql.extend(nullable_bigint_sql("dashboard_recent_error_snapshots", "first_byte_ms", "ttfb_ms"));
    sql.push("UPDATE dashboard_latency_histogram_buckets SET metric_kind = 'first_byte' WHERE metric_kind = 'ttfb'".to_owned());
    sql
}

fn snapshot_json_sql() -> Vec<String> {
    [
        ("\"p50_ttfb_ms\":", "\"p50_first_byte_ms\":"),
        ("\"p90_ttfb_ms\":", "\"p90_first_byte_ms\":"),
        ("\"p95_ttfb_ms\":", "\"p95_first_byte_ms\":"),
        ("\"p99_ttfb_ms\":", "\"p99_first_byte_ms\":"),
        ("\"p50_first_output_ms\":", "\"p50_first_token_ms\":"),
        ("\"p90_first_output_ms\":", "\"p90_first_token_ms\":"),
        ("\"p95_first_output_ms\":", "\"p95_first_token_ms\":"),
        ("\"p99_first_output_ms\":", "\"p99_first_token_ms\":"),
    ]
    .into_iter()
    .map(|(old_value, new_value)| replace_json_key_sql("performance_monitoring_snapshots", "metrics", old_value, new_value))
    .collect()
}

fn nullable_bigint_sql(table: &str, new_column: &str, old_column: &str) -> [String; 3] {
    [
        format!("ALTER TABLE IF EXISTS {table} ADD COLUMN IF NOT EXISTS {new_column} BIGINT NULL"),
        format!("UPDATE {table} SET {new_column} = {old_column} WHERE {new_column} IS NULL AND {old_column} IS NOT NULL"),
        format!("ALTER TABLE IF EXISTS {table} DROP COLUMN IF EXISTS {old_column}"),
    ]
}

fn counter_sql(table: &str, new_column: &str, old_column: &str) -> [String; 3] {
    [
        format!("ALTER TABLE IF EXISTS {table} ADD COLUMN IF NOT EXISTS {new_column} BIGINT NOT NULL DEFAULT 0"),
        format!("UPDATE {table} SET {new_column} = {old_column} WHERE {new_column} = 0 AND {old_column} <> 0"),
        format!("ALTER TABLE IF EXISTS {table} DROP COLUMN IF EXISTS {old_column}"),
    ]
}

fn replace_json_key_sql(table: &str, column: &str, old_value: &str, new_value: &str) -> String {
    format!(
        "UPDATE {table} SET {column} = REPLACE({column}, '{old_value}', '{new_value}') \
         WHERE {column} IS NOT NULL AND POSITION('{old_value}' IN {column}) > 0"
    )
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: impl Into<String>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_raw(Statement::from_string(manager.get_database_backend(), sql.into()))
        .await?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::{column_semantics_sql, snapshot_json_sql};

    #[test]
    fn timing_columns_move_ttfb_to_first_byte() {
        let sql = column_semantics_sql().join(" ");

        assert!(sql.contains("dashboard_request_metric_buckets ADD COLUMN IF NOT EXISTS first_byte_total_ms"));
        assert!(sql.contains("dashboard_recent_error_snapshots ADD COLUMN IF NOT EXISTS first_byte_ms"));
        assert!(sql.contains("metric_kind = 'first_byte' WHERE metric_kind = 'ttfb'"));
    }

    #[test]
    fn snapshot_metrics_keys_move_to_first_byte_and_first_token() {
        let sql = snapshot_json_sql().join(" ");

        assert!(sql.contains("\"p50_ttfb_ms\":', '\"p50_first_byte_ms\":'"));
        assert!(sql.contains("\"p99_first_output_ms\":', '\"p99_first_token_ms\":'"));
    }
}
