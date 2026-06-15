use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260614_000001_routing_metrics";
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

fn table_sql() -> [&'static str; 5] {
    [
        metric_table_sql(),
        route_state_table_sql(),
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
     latency_sum_ms BIGINT NOT NULL, latency_sample_count BIGINT NOT NULL, ttfb_sum_ms BIGINT NOT NULL, ttfb_sample_count BIGINT NOT NULL, \
     output_tokens BIGINT NOT NULL, tps_latency_sum_ms BIGINT NOT NULL, tps_sample_count BIGINT NOT NULL, upstream_total_cost DECIMAL(20, 8) NOT NULL, \
     total_tokens BIGINT NOT NULL, last_seen_at TIMESTAMPTZ NOT NULL, created_at TIMESTAMPTZ NOT NULL, updated_at TIMESTAMPTZ NOT NULL)"
}

fn route_state_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS routing_route_states (\
     provider_id VARCHAR(36) NOT NULL, key_id VARCHAR(36) NOT NULL, endpoint_id VARCHAR(36) NOT NULL, global_model_id VARCHAR(36) NOT NULL, \
     client_api_format VARCHAR(50) NOT NULL, provider_api_format VARCHAR(50) NOT NULL, is_stream BOOLEAN NOT NULL, ema_success_rate DECIMAL(20, 8) NOT NULL, \
     ema_ttfb_ms DECIMAL(20, 8) NULL, ema_latency_ms DECIMAL(20, 8) NULL, ema_output_tps DECIMAL(20, 8) NULL, learned_rpm_limit INTEGER NULL, \
     sample_count BIGINT NOT NULL, state VARCHAR(32) NOT NULL, last_updated_at TIMESTAMPTZ NOT NULL, \
     PRIMARY KEY (provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream))"
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

fn index_sql() -> [&'static str; 6] {
    [
        "CREATE UNIQUE INDEX IF NOT EXISTS index_routing_metric_buckets_unique ON routing_metric_buckets \
         (bucket_granularity, bucket_started_at, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream)",
        "CREATE INDEX IF NOT EXISTS index_routing_metric_buckets_by_window ON routing_metric_buckets (bucket_granularity, bucket_started_at)",
        "CREATE INDEX IF NOT EXISTS index_routing_route_states_by_updated ON routing_route_states (last_updated_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_routing_decision_samples_created ON routing_decision_samples (created_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_routing_profiles_updated ON routing_profiles (updated_at DESC)",
        "CREATE INDEX IF NOT EXISTS index_routing_profile_versions_created ON routing_profile_versions (profile_id, created_at DESC)",
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

#[cfg(test)]
mod tests {
    use super::{decision_sample_table_sql, metric_table_sql, profile_version_table_sql, route_state_table_sql};

    #[test]
    fn metric_table_is_key_level() {
        let sql = metric_table_sql();

        assert!(sql.contains("key_id VARCHAR(36) NOT NULL"));
        assert!(sql.contains("endpoint_id VARCHAR(36) NOT NULL"));
    }

    #[test]
    fn route_state_table_keeps_ema_fields() {
        let sql = route_state_table_sql();

        assert!(sql.contains("ema_success_rate"));
        assert!(sql.contains("ema_ttfb_ms"));
    }

    #[test]
    fn decision_samples_store_score_explanations() {
        assert!(decision_sample_table_sql().contains("candidate_scores TEXT NOT NULL"));
    }

    #[test]
    fn profile_versions_keep_effective_weights() {
        assert!(profile_version_table_sql().contains("effective_weights TEXT NOT NULL"));
    }
}
