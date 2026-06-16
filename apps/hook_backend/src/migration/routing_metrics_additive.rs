use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260614_000001_routing_metrics";
const FINGERPRINT_VERSION: &str = "m20260616_000001_routing_fingerprints";
const MIGRATION_TABLE: &str = "seaql_migrations";
const LEGACY_FINGERPRINT: &str = "legacy";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let additive_exists = migration_marker_exists(manager, ADDITIVE_VERSION).await?;
    if !additive_exists {
        create_tables(manager).await?;
    }
    ensure_v2_tables(manager).await?;
    ensure_quality_metric_columns(manager).await?;
    if !migration_marker_exists(manager, FINGERPRINT_VERSION).await? {
        ensure_fingerprint_columns(manager).await?;
        ensure_fingerprint_keys(manager).await?;
        create_indexes(manager).await?;
        mark_migration_applied(manager, FINGERPRINT_VERSION).await?;
    }
    if !additive_exists {
        mark_migration_applied(manager, ADDITIVE_VERSION).await?;
    }
    Ok(())
}

async fn create_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in table_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in index_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn ensure_v2_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in v2_table_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

fn table_sql() -> [&'static str; 6] {
    [
        metric_table_sql(),
        route_state_table_sql(),
        context_route_state_table_sql(),
        decision_sample_table_sql(),
        profile_table_sql(),
        profile_version_table_sql(),
    ]
}

fn metric_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS routing_metric_buckets (\
     id VARCHAR(36) PRIMARY KEY, bucket_granularity VARCHAR(16) NOT NULL, bucket_started_at TIMESTAMPTZ NOT NULL, bucket_ended_at TIMESTAMPTZ NOT NULL, \
     provider_id VARCHAR(36) NOT NULL, provider_name VARCHAR(100) NULL, key_id VARCHAR(36) NOT NULL, key_name VARCHAR(100) NULL, \
     endpoint_id VARCHAR(36) NOT NULL, endpoint_name VARCHAR(50) NULL, global_model_id VARCHAR(36) NOT NULL, \
     client_api_format VARCHAR(50) NOT NULL, provider_api_format VARCHAR(50) NOT NULL, is_stream BOOLEAN NOT NULL, request_count BIGINT NOT NULL, \
     success_count BIGINT NOT NULL, failure_count BIGINT NOT NULL, timeout_count BIGINT NOT NULL, rate_limited_count BIGINT NOT NULL, server_error_count BIGINT NOT NULL, \
     format_conversion_failure_count BIGINT NOT NULL DEFAULT 0, usage_missing_count BIGINT NOT NULL DEFAULT 0, stream_abnormal_end_count BIGINT NOT NULL DEFAULT 0, \
     schema_tool_call_failure_count BIGINT NOT NULL DEFAULT 0, \
     latency_sum_ms BIGINT NOT NULL, latency_sample_count BIGINT NOT NULL, ttfb_sum_ms BIGINT NOT NULL, ttfb_sample_count BIGINT NOT NULL, \
     output_tokens BIGINT NOT NULL, tps_latency_sum_ms BIGINT NOT NULL, tps_sample_count BIGINT NOT NULL, upstream_total_cost DECIMAL(20, 8) NOT NULL, \
     total_tokens BIGINT NOT NULL, route_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy', price_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy', \
     last_seen_at TIMESTAMPTZ NOT NULL, created_at TIMESTAMPTZ NOT NULL, updated_at TIMESTAMPTZ NOT NULL)"
}

fn route_state_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS routing_route_states (\
     provider_id VARCHAR(36) NOT NULL, key_id VARCHAR(36) NOT NULL, endpoint_id VARCHAR(36) NOT NULL, global_model_id VARCHAR(36) NOT NULL, \
     client_api_format VARCHAR(50) NOT NULL, provider_api_format VARCHAR(50) NOT NULL, is_stream BOOLEAN NOT NULL, \
     route_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy', price_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy', ema_success_rate DECIMAL(20, 8) NOT NULL, \
     ema_ttfb_ms DECIMAL(20, 8) NULL, ema_latency_ms DECIMAL(20, 8) NULL, ema_output_tps DECIMAL(20, 8) NULL, learned_rpm_limit INTEGER NULL, \
     sample_count BIGINT NOT NULL, state VARCHAR(32) NOT NULL, last_updated_at TIMESTAMPTZ NOT NULL, \
     PRIMARY KEY (provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint))"
}

fn context_route_state_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS routing_context_route_states (\
     context_key VARCHAR(255) NOT NULL, provider_id VARCHAR(36) NOT NULL, key_id VARCHAR(36) NOT NULL, endpoint_id VARCHAR(36) NOT NULL, \
     global_model_id VARCHAR(36) NOT NULL, client_api_format VARCHAR(50) NOT NULL, provider_api_format VARCHAR(50) NOT NULL, is_stream BOOLEAN NOT NULL, \
     route_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy', price_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy', sample_count BIGINT NOT NULL, success_count BIGINT NOT NULL, \
     failure_count BIGINT NOT NULL, ema_success_rate DECIMAL(20, 8) NOT NULL, ema_ttfb_ms DECIMAL(20, 8) NULL, ema_latency_ms DECIMAL(20, 8) NULL, \
     ema_output_tps DECIMAL(20, 8) NULL, last_updated_at TIMESTAMPTZ NOT NULL, \
     PRIMARY KEY (context_key, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint))"
}

fn decision_sample_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS routing_decision_samples (\
     request_id VARCHAR(64) PRIMARY KEY, profile_id VARCHAR(64) NOT NULL, profile_version VARCHAR(64) NOT NULL, selected_route TEXT NULL, \
     candidate_scores TEXT NOT NULL, exclusion_reasons TEXT NOT NULL, created_at TIMESTAMPTZ NOT NULL)"
}

fn profile_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS routing_profiles (\
     profile_id VARCHAR(64) PRIMARY KEY, profile_version VARCHAR(64) NOT NULL, profile_config TEXT NOT NULL, updated_at TIMESTAMPTZ NOT NULL)"
}

fn profile_version_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS routing_profile_versions (\
     id VARCHAR(36) PRIMARY KEY, profile_id VARCHAR(64) NOT NULL, profile_version VARCHAR(64) NOT NULL, admin_weights TEXT NOT NULL, \
     learned_weights TEXT NULL, effective_weights TEXT NOT NULL, reward_window VARCHAR(16) NOT NULL, sample_count BIGINT NOT NULL, created_at TIMESTAMPTZ NOT NULL)"
}

fn v2_table_sql() -> [&'static str; 1] {
    [context_route_state_table_sql()]
}

fn index_sql() -> [&'static str; 8] {
    [
        "DROP INDEX IF EXISTS index_routing_metric_buckets_unique",
        "CREATE UNIQUE INDEX IF NOT EXISTS index_routing_metric_buckets_unique ON routing_metric_buckets \
         (bucket_granularity, bucket_started_at, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, \
         route_config_fingerprint, price_config_fingerprint)",
        "CREATE INDEX IF NOT EXISTS index_routing_metric_buckets_by_window ON routing_metric_buckets (bucket_granularity, bucket_started_at)",
        "CREATE INDEX IF NOT EXISTS index_routing_route_states_by_updated ON routing_route_states (last_updated_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_routing_context_route_states_by_updated ON routing_context_route_states (context_key, last_updated_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_routing_decision_samples_created ON routing_decision_samples (created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_routing_profiles_updated ON routing_profiles (updated_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_routing_profile_versions_created ON routing_profile_versions (profile_id, created_at DESC)",
    ]
}

async fn ensure_quality_metric_columns(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in quality_metric_column_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn ensure_fingerprint_columns(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in fingerprint_column_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn ensure_fingerprint_keys(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in fingerprint_key_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

fn quality_metric_column_sql() -> [&'static str; 4] {
    [
        "ALTER TABLE IF EXISTS routing_metric_buckets ADD COLUMN IF NOT EXISTS format_conversion_failure_count BIGINT NOT NULL DEFAULT 0",
        "ALTER TABLE IF EXISTS routing_metric_buckets ADD COLUMN IF NOT EXISTS usage_missing_count BIGINT NOT NULL DEFAULT 0",
        "ALTER TABLE IF EXISTS routing_metric_buckets ADD COLUMN IF NOT EXISTS stream_abnormal_end_count BIGINT NOT NULL DEFAULT 0",
        "ALTER TABLE IF EXISTS routing_metric_buckets ADD COLUMN IF NOT EXISTS schema_tool_call_failure_count BIGINT NOT NULL DEFAULT 0",
    ]
}

fn fingerprint_column_sql() -> Vec<String> {
    ["routing_metric_buckets", "routing_route_states", "routing_context_route_states"]
        .into_iter()
        .flat_map(fingerprint_table_column_sql)
        .collect()
}

fn fingerprint_table_column_sql(table: &str) -> [String; 8] {
    [
        format!("ALTER TABLE IF EXISTS {table} ADD COLUMN IF NOT EXISTS route_config_fingerprint VARCHAR(64) NULL"),
        format!("ALTER TABLE IF EXISTS {table} ADD COLUMN IF NOT EXISTS price_config_fingerprint VARCHAR(64) NULL"),
        format!("UPDATE {table} SET route_config_fingerprint = '{LEGACY_FINGERPRINT}' WHERE route_config_fingerprint IS NULL"),
        format!("UPDATE {table} SET price_config_fingerprint = '{LEGACY_FINGERPRINT}' WHERE price_config_fingerprint IS NULL"),
        format!("ALTER TABLE IF EXISTS {table} ALTER COLUMN route_config_fingerprint SET DEFAULT '{LEGACY_FINGERPRINT}'"),
        format!("ALTER TABLE IF EXISTS {table} ALTER COLUMN price_config_fingerprint SET DEFAULT '{LEGACY_FINGERPRINT}'"),
        format!("ALTER TABLE IF EXISTS {table} ALTER COLUMN route_config_fingerprint SET NOT NULL"),
        format!("ALTER TABLE IF EXISTS {table} ALTER COLUMN price_config_fingerprint SET NOT NULL"),
    ]
}

fn fingerprint_key_sql() -> [&'static str; 4] {
    [
        "ALTER TABLE IF EXISTS routing_route_states DROP CONSTRAINT IF EXISTS routing_route_states_pkey",
        "ALTER TABLE IF EXISTS routing_route_states ADD CONSTRAINT routing_route_states_pkey PRIMARY KEY \
         (provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint)",
        "ALTER TABLE IF EXISTS routing_context_route_states DROP CONSTRAINT IF EXISTS routing_context_route_states_pkey",
        "ALTER TABLE IF EXISTS routing_context_route_states ADD CONSTRAINT routing_context_route_states_pkey PRIMARY KEY \
         (context_key, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint)",
    ]
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: impl Into<String>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_raw(Statement::from_string(manager.get_database_backend(), sql.into()))
        .await?;
    Ok(())
}

async fn migration_marker_exists(manager: &SchemaManager<'_>, version: &str) -> Result<bool, DbErr> {
    if !manager.has_table(MIGRATION_TABLE).await? {
        return Ok(false);
    }
    seaql_migrations::Entity::find()
        .filter(seaql_migrations::Column::Version.eq(version))
        .one(manager.get_connection())
        .await
        .map(|record| record.is_some())
}

async fn mark_migration_applied(manager: &SchemaManager<'_>, version: &str) -> Result<(), DbErr> {
    create_migration_table(manager).await?;
    if migration_marker_exists(manager, version).await? {
        return Ok(());
    }
    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(version.to_owned()),
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
mod tests;
