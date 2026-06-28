use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260609_000005_dashboard_request_metrics";
const MIGRATION_TABLE: &str = "seaql_migrations";
const REQUEST_METRIC_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS dashboard_request_metric_buckets (\
 id VARCHAR(36) PRIMARY KEY, \
 source_type VARCHAR(16) NOT NULL, \
 bucket_granularity VARCHAR(16) NOT NULL, \
 bucket_started_at TIMESTAMPTZ NOT NULL, \
 bucket_ended_at TIMESTAMPTZ NOT NULL, \
 user_id VARCHAR(36) NULL, \
 username VARCHAR(100) NULL, \
 token_id VARCHAR(36) NULL, \
 token_name VARCHAR(100) NULL, \
 token_prefix VARCHAR(32) NULL, \
 provider_id VARCHAR(36) NULL, \
 provider_name VARCHAR(100) NULL, \
 global_model_id VARCHAR(100) NULL, \
 model_name VARCHAR(100) NULL, \
 client_api_format VARCHAR(50) NULL, \
 provider_api_format VARCHAR(50) NULL, \
 is_stream BOOLEAN NULL, \
 needs_conversion BOOLEAN NULL, \
 request_count BIGINT NOT NULL, \
 success_count BIGINT NOT NULL, \
 failed_count BIGINT NOT NULL, \
 active_count BIGINT NOT NULL, \
 prompt_tokens BIGINT NOT NULL, \
 completion_tokens BIGINT NOT NULL, \
 cache_creation_input_tokens BIGINT NOT NULL, \
 cache_read_input_tokens BIGINT NOT NULL, \
 total_tokens BIGINT NOT NULL, \
 output_tokens BIGINT NOT NULL, \
 total_cost DECIMAL(20, 8) NOT NULL, \
 base_cost DECIMAL(20, 8) NOT NULL, \
 upstream_total_cost DECIMAL(20, 8) NOT NULL, \
 cache_read_cost DECIMAL(20, 8) NOT NULL, \
 cache_creation_cost DECIMAL(20, 8) NOT NULL, \
 latency_total_ms BIGINT NOT NULL, \
 latency_sample_count BIGINT NOT NULL, \
 ttfb_total_ms BIGINT NOT NULL, \
 ttfb_sample_count BIGINT NOT NULL, \
 response_headers_total_ms BIGINT NOT NULL DEFAULT 0, \
 response_headers_sample_count BIGINT NOT NULL DEFAULT 0, \
 first_sse_event_total_ms BIGINT NOT NULL DEFAULT 0, \
 first_sse_event_sample_count BIGINT NOT NULL DEFAULT 0, \
 first_output_total_ms BIGINT NOT NULL DEFAULT 0, \
 first_output_sample_count BIGINT NOT NULL DEFAULT 0, \
 sse_to_output_total_ms BIGINT NOT NULL DEFAULT 0, \
 sse_to_output_sample_count BIGINT NOT NULL DEFAULT 0, \
 tps_latency_total_ms BIGINT NOT NULL, \
 tps_output_tokens BIGINT NOT NULL, \
 tps_sample_count BIGINT NOT NULL, \
 retry_count BIGINT NOT NULL, \
 failover_count BIGINT NOT NULL, \
 timeout_count BIGINT NOT NULL, \
 rate_limited_count BIGINT NOT NULL, \
 server_error_count BIGINT NOT NULL, \
 quota_limited_count BIGINT NOT NULL, \
 slow_request_count BIGINT NOT NULL, \
 created_at TIMESTAMPTZ NOT NULL, \
 updated_at TIMESTAMPTZ NOT NULL)";

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

fn table_sql() -> [&'static str; 4] {
    [
        request_metric_table_sql(),
        latency_histogram_table_sql(),
        recent_error_table_sql(),
        metric_sync_state_table_sql(),
    ]
}

fn request_metric_table_sql() -> &'static str {
    REQUEST_METRIC_TABLE_SQL
}

fn latency_histogram_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS dashboard_latency_histogram_buckets (\
     id VARCHAR(36) PRIMARY KEY, \
     source_type VARCHAR(16) NOT NULL, \
     metric_kind VARCHAR(16) NOT NULL, \
     bucket_granularity VARCHAR(16) NOT NULL, \
     bucket_started_at TIMESTAMPTZ NOT NULL, \
     bucket_ended_at TIMESTAMPTZ NOT NULL, \
     le_ms BIGINT NOT NULL, \
     provider_id VARCHAR(36) NULL, \
     provider_name VARCHAR(100) NULL, \
     global_model_id VARCHAR(100) NULL, \
     provider_api_format VARCHAR(50) NULL, \
     is_stream BOOLEAN NULL, \
     needs_conversion BOOLEAN NULL, \
     sample_count BIGINT NOT NULL, \
     created_at TIMESTAMPTZ NOT NULL, \
     updated_at TIMESTAMPTZ NOT NULL)"
}

fn recent_error_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS dashboard_recent_error_snapshots (\
     request_id VARCHAR(64) PRIMARY KEY, \
     created_at TIMESTAMPTZ NOT NULL, \
     provider_id VARCHAR(36) NULL, \
     provider_name VARCHAR(100) NULL, \
     model VARCHAR(100) NULL, \
     status_code INTEGER NULL, \
     error_type VARCHAR(100) NULL, \
     error_message TEXT NULL, \
     error_category VARCHAR(100) NOT NULL, \
     latency_ms BIGINT NULL, \
     ttfb_ms BIGINT NULL, \
     updated_at TIMESTAMPTZ NOT NULL)"
}

fn metric_sync_state_table_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS dashboard_request_metric_sync_states (\
     owner_type VARCHAR(16) NOT NULL, \
     owner_id VARCHAR(64) NOT NULL, \
     created_at TIMESTAMPTZ NOT NULL, \
     updated_at TIMESTAMPTZ NOT NULL, \
     PRIMARY KEY (owner_type, owner_id))"
}

fn index_sql() -> [&'static str; 7] {
    [
        "CREATE UNIQUE INDEX IF NOT EXISTS index_dashboard_request_metric_buckets_unique ON dashboard_request_metric_buckets \
         (source_type, bucket_granularity, bucket_started_at, user_id, token_id, provider_id, global_model_id, client_api_format, provider_api_format, is_stream, needs_conversion) NULLS NOT DISTINCT",
        "CREATE INDEX IF NOT EXISTS index_dashboard_request_metric_buckets_by_bucket ON dashboard_request_metric_buckets (source_type, bucket_granularity, bucket_started_at)",
        "CREATE INDEX IF NOT EXISTS index_dashboard_request_metric_buckets_by_user_bucket ON dashboard_request_metric_buckets (source_type, user_id, bucket_granularity, bucket_started_at)",
        "CREATE INDEX IF NOT EXISTS index_dashboard_request_metric_buckets_by_token_bucket ON dashboard_request_metric_buckets (source_type, token_id, bucket_granularity, bucket_started_at)",
        "CREATE UNIQUE INDEX IF NOT EXISTS index_dashboard_latency_histogram_buckets_unique ON dashboard_latency_histogram_buckets \
         (source_type, metric_kind, bucket_granularity, bucket_started_at, le_ms, provider_id, global_model_id, provider_api_format, is_stream, needs_conversion) NULLS NOT DISTINCT",
        "CREATE INDEX IF NOT EXISTS index_dashboard_latency_histogram_buckets_by_bucket ON dashboard_latency_histogram_buckets (source_type, metric_kind, bucket_granularity, bucket_started_at, le_ms)",
        "CREATE INDEX IF NOT EXISTS index_dashboard_recent_error_snapshots_by_created ON dashboard_recent_error_snapshots (created_at DESC, request_id DESC)",
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
    use super::{index_sql, latency_histogram_table_sql, metric_sync_state_table_sql, recent_error_table_sql, request_metric_table_sql};

    #[test]
    fn metric_table_keeps_request_and_candidate_dimensions() {
        assert!(request_metric_table_sql().contains("source_type VARCHAR(16) NOT NULL"));
        assert!(request_metric_table_sql().contains("needs_conversion BOOLEAN NULL"));
    }

    #[test]
    fn histogram_unique_index_treats_null_dimensions_as_equal() {
        let unique = index_sql()[4];

        assert!(latency_histogram_table_sql().contains("le_ms BIGINT NOT NULL"));
        assert!(unique.contains("NULLS NOT DISTINCT"));
    }

    #[test]
    fn recent_error_snapshots_are_keyed_by_request_id() {
        assert!(recent_error_table_sql().contains("request_id VARCHAR(64) PRIMARY KEY"));
        assert!(recent_error_table_sql().contains("error_category VARCHAR(100) NOT NULL"));
    }

    #[test]
    fn metric_sync_states_track_applied_request_boundaries() {
        assert!(metric_sync_state_table_sql().contains("PRIMARY KEY (owner_type, owner_id)"));
    }
}
