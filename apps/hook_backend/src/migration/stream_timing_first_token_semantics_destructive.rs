use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const DESTRUCTIVE_VERSION: &str = "m20260630_000002_stream_timing_first_token_semantics";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if destructive_marker_exists(manager).await? {
        return Ok(());
    }
    for sql in column_semantics_sql() {
        execute_sql(manager, sql).await?;
    }
    for sql in routing_profile_sql() {
        execute_sql(manager, sql).await?;
    }
    mark_destructive_applied(manager).await
}

fn column_semantics_sql() -> Vec<String> {
    let mut sql = Vec::new();
    sql.extend(nullable_bigint_sql("request_records", "first_token_time_ms", "first_output_time_ms"));
    sql.extend(nullable_bigint_sql(
        "request_records_partitioned",
        "first_token_time_ms",
        "first_output_time_ms",
    ));
    sql.extend(nullable_bigint_sql("request_candidates", "first_token_time_ms", "first_output_time_ms"));
    sql.extend(nullable_bigint_sql(
        "request_candidates_partitioned",
        "first_token_time_ms",
        "first_output_time_ms",
    ));
    sql.extend(counter_sql("dashboard_request_metric_buckets", "first_token_total_ms", "first_output_total_ms"));
    sql.extend(counter_sql(
        "dashboard_request_metric_buckets",
        "first_token_sample_count",
        "first_output_sample_count",
    ));
    sql.extend(counter_sql("dashboard_cost_analysis_buckets", "first_token_total_ms", "first_output_total_ms"));
    sql.extend(counter_sql(
        "dashboard_cost_analysis_buckets",
        "first_token_sample_count",
        "first_output_sample_count",
    ));
    sql.extend(counter_sql("dashboard_user_usage_buckets", "first_token_total_ms", "first_output_total_ms"));
    sql.extend(counter_sql(
        "dashboard_user_usage_buckets",
        "first_token_sample_count",
        "first_output_sample_count",
    ));
    sql.extend(nullable_bigint_sql("dashboard_recent_error_snapshots", "first_token_ms", "first_output_ms"));
    sql.push("UPDATE dashboard_latency_histogram_buckets SET metric_kind = 'first_token' WHERE metric_kind = 'first_output'".to_owned());
    sql.extend(counter_sql("routing_metric_buckets", "first_token_success_count", "first_output_success_count"));
    sql.extend(counter_sql("routing_metric_buckets", "first_token_failure_count", "first_output_failure_count"));
    sql.extend(counter_sql("routing_metric_buckets", "first_token_sum_ms", "ttfb_sum_ms"));
    sql.extend(counter_sql("routing_metric_buckets", "first_token_sample_count", "ttfb_sample_count"));
    sql.extend(nullable_decimal_sql("routing_route_states", "ema_first_token_ms", "ema_ttfb_ms"));
    sql.extend(nullable_decimal_sql("routing_context_route_states", "ema_first_token_ms", "ema_ttfb_ms"));
    sql
}

fn routing_profile_sql() -> Vec<String> {
    vec![
        replace_json_key_sql("routing_profiles", "profile_config", "\"id\":\"first_byte\"", "\"id\":\"first_token\""),
        replace_json_key_sql("routing_profiles", "profile_config", "\"ttfb\":", "\"first_token\":"),
        replace_json_key_sql("routing_profile_versions", "admin_weights", "\"ttfb\":", "\"first_token\":"),
        replace_json_key_sql("routing_profile_versions", "learned_weights", "\"ttfb\":", "\"first_token\":"),
        replace_json_key_sql("routing_profile_versions", "effective_weights", "\"ttfb\":", "\"first_token\":"),
        replace_json_key_sql("routing_decision_samples", "candidate_scores", "\"ttfb_avg_ms\":", "\"first_token_avg_ms\":"),
        replace_json_key_sql("routing_decision_samples", "candidate_scores", "\"code\":\"ttfb\"", "\"code\":\"first_token\""),
        update_profile_id_sql("routing_profiles", "profile_id"),
        update_profile_id_sql("routing_profile_versions", "profile_id"),
        update_profile_id_sql("routing_decision_samples", "profile_id"),
        update_profile_id_sql("routing_route_states", "profile_id"),
        update_profile_id_sql("routing_context_route_states", "profile_id"),
        update_profile_id_sql("billing_groups", "routing_profile_id"),
        update_profile_id_sql("global_models", "routing_profile_id"),
    ]
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

fn nullable_decimal_sql(table: &str, new_column: &str, old_column: &str) -> [String; 3] {
    [
        format!("ALTER TABLE IF EXISTS {table} ADD COLUMN IF NOT EXISTS {new_column} DECIMAL(20, 8) NULL"),
        format!("UPDATE {table} SET {new_column} = {old_column} WHERE {new_column} IS NULL AND {old_column} IS NOT NULL"),
        format!("ALTER TABLE IF EXISTS {table} DROP COLUMN IF EXISTS {old_column}"),
    ]
}

fn replace_json_key_sql(table: &str, column: &str, old_value: &str, new_value: &str) -> String {
    format!(
        "UPDATE {table} SET {column} = REPLACE({column}, '{old_value}', '{new_value}') \
         WHERE {column} IS NOT NULL AND POSITION('{old_value}' IN {column}) > 0"
    )
}

fn update_profile_id_sql(table: &str, column: &str) -> String {
    format!("UPDATE {table} SET {column} = 'first_token' WHERE {column} = 'first_byte'")
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
    use super::{column_semantics_sql, routing_profile_sql};

    #[test]
    fn timing_columns_move_first_output_to_first_token() {
        let sql = column_semantics_sql().join(" ");

        assert!(sql.contains("request_records ADD COLUMN IF NOT EXISTS first_token_time_ms"));
        assert!(sql.contains("dashboard_request_metric_buckets ADD COLUMN IF NOT EXISTS first_token_total_ms"));
        assert!(sql.contains("routing_metric_buckets ADD COLUMN IF NOT EXISTS first_token_sum_ms"));
        assert!(sql.contains("routing_route_states ADD COLUMN IF NOT EXISTS ema_first_token_ms"));
        assert!(sql.contains("metric_kind = 'first_token' WHERE metric_kind = 'first_output'"));
    }

    #[test]
    fn routing_profiles_and_samples_upgrade_legacy_first_byte_payloads() {
        let sql = routing_profile_sql().join(" ");

        assert!(sql.contains("profile_config = REPLACE(profile_config, '\"id\":\"first_byte\"', '\"id\":\"first_token\"')"));
        assert!(sql.contains("candidate_scores = REPLACE(candidate_scores, '\"ttfb_avg_ms\":', '\"first_token_avg_ms\":')"));
        assert!(sql.contains("SET profile_id = 'first_token' WHERE profile_id = 'first_byte'"));
        assert!(sql.contains("SET routing_profile_id = 'first_token' WHERE routing_profile_id = 'first_byte'"));
    }
}
