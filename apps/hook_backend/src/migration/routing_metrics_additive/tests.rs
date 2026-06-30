use types::provider::{ROUTING_TIMING_SEMANTICS_COLUMN, ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1};

use super::{
    context_route_state_table_sql, decision_sample_table_sql, fingerprint_key_sql, index_sql, metric_table_sql, profile_version_table_sql,
    quality_metric_column_sql, route_state_table_sql, v2_table_sql,
};

#[test]
fn metric_table_is_key_level() {
    let sql = metric_table_sql();

    assert!(sql.contains("key_id VARCHAR(36) NOT NULL"));
    assert!(sql.contains("endpoint_id VARCHAR(36) NOT NULL"));
    assert!(sql.contains("route_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy'"));
    assert!(sql.contains("price_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy'"));
    assert!(sql.contains("format_conversion_failure_count BIGINT NOT NULL DEFAULT 0"));
    assert!(sql.contains(&format!(
        "{ROUTING_TIMING_SEMANTICS_COLUMN} VARCHAR(32) NOT NULL DEFAULT '{ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1}'"
    )));
}

#[test]
fn metric_quality_columns_are_additive_for_existing_tables() {
    let sql = quality_metric_column_sql().join(" ");

    assert!(sql.contains("ADD COLUMN IF NOT EXISTS usage_missing_count"));
    assert!(sql.contains("ADD COLUMN IF NOT EXISTS stream_abnormal_end_count"));
}

#[test]
fn route_state_table_keeps_ema_fields() {
    let sql = route_state_table_sql();

    assert!(sql.contains("ema_success_rate"));
    assert!(sql.contains("ema_ttfb_ms"));
    assert!(sql.contains("route_config_fingerprint VARCHAR(64) NOT NULL DEFAULT 'legacy'"));
    assert!(sql.contains("route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version"));
}

#[test]
fn context_route_state_table_is_keyed_by_context_and_route() {
    let sql = context_route_state_table_sql();

    assert!(sql.contains("context_key VARCHAR(255) NOT NULL"));
    assert!(sql.contains("PRIMARY KEY (context_key, provider_id, key_id, endpoint_id"));
    assert!(sql.contains("route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version"));
}

#[test]
fn v2_table_sql_backfills_tables_added_after_initial_marker() {
    let sql = v2_table_sql().join(" ");

    assert!(sql.contains("CREATE TABLE IF NOT EXISTS routing_context_route_states"));
}

#[test]
fn fingerprint_indexes_and_keys_include_fingerprints() {
    let index_sql = index_sql().join(" ");
    let key_sql = fingerprint_key_sql().join(" ");

    assert!(index_sql.contains("DROP INDEX IF EXISTS index_routing_metric_buckets_unique"));
    assert!(index_sql.contains("route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version"));
    assert!(key_sql.contains("routing_route_states_pkey"));
    assert!(key_sql.contains("routing_context_route_states_pkey"));
}

#[test]
fn decision_samples_store_score_explanations() {
    assert!(decision_sample_table_sql().contains("candidate_scores TEXT NOT NULL"));
}

#[test]
fn profile_versions_keep_effective_weights() {
    assert!(profile_version_table_sql().contains("effective_weights TEXT NOT NULL"));
}
