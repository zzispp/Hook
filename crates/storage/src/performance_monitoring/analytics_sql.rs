use sea_orm::Value;
use types::performance_monitoring::{PerformanceMonitoringAnalyticsRequest, SnapshotGranularity};

use super::types::SnapshotQueryPlan;

pub(super) const DEFAULT_ANALYTICS_LIMIT: usize = 8;
pub(super) const MAX_ANALYTICS_LIMIT: usize = 20;
pub(super) const DEFAULT_SLOW_THRESHOLD_MS: i64 = 10_000;
pub(super) const RECENT_ERROR_LIMIT: i64 = 12;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UpstreamFilters {
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub api_format: Option<String>,
    pub is_stream: Option<bool>,
    pub needs_conversion: Option<bool>,
}

impl UpstreamFilters {
    pub(super) fn from_request(request: &PerformanceMonitoringAnalyticsRequest) -> Self {
        Self {
            provider_id: normalized_string(request.provider_id.clone()),
            model: normalized_string(request.model.clone()),
            api_format: normalized_string(request.api_format.clone()),
            is_stream: request.is_stream,
            needs_conversion: request.needs_conversion,
        }
    }
}

pub(super) struct SqlParts {
    pub sql: String,
    pub values: Vec<Value>,
}

pub(super) fn plan_values(plan: &SnapshotQueryPlan) -> Vec<Value> {
    vec![Value::from(plan.started_at), Value::from(plan.ended_at)]
}

pub(super) fn percentile_sql(granularity: SnapshotGranularity) -> String {
    let bucket = bucket_expr("created_at", granularity);
    let bucket_end = bucket_end_expr("bucket_started_at", granularity);
    format!(
        "WITH filtered AS ( \
            SELECT {bucket} AS bucket_started_at, total_latency_ms, first_byte_time_ms \
            FROM request_records \
            WHERE created_at >= $1 AND created_at < $2 AND status = 'success' \
                AND (total_latency_ms IS NOT NULL OR first_byte_time_ms IS NOT NULL) \
        ) \
        SELECT bucket_started_at, {bucket_end} AS bucket_ended_at, \
            percentile_disc(0.50) WITHIN GROUP (ORDER BY total_latency_ms) AS p50_latency_ms, \
            percentile_disc(0.90) WITHIN GROUP (ORDER BY total_latency_ms) AS p90_latency_ms, \
            percentile_disc(0.99) WITHIN GROUP (ORDER BY total_latency_ms) AS p99_latency_ms, \
            percentile_disc(0.50) WITHIN GROUP (ORDER BY first_byte_time_ms) AS p50_ttfb_ms, \
            percentile_disc(0.90) WITHIN GROUP (ORDER BY first_byte_time_ms) AS p90_ttfb_ms, \
            percentile_disc(0.99) WITHIN GROUP (ORDER BY first_byte_time_ms) AS p99_ttfb_ms \
        FROM filtered GROUP BY bucket_started_at ORDER BY bucket_started_at ASC"
    )
}

pub(super) fn error_distribution_sql() -> String {
    format!(
        "SELECT {category} AS category, COUNT(*)::bigint AS count \
        FROM request_records \
        WHERE created_at >= $1 AND created_at < $2 AND status IN ('failed', 'cancelled') \
        GROUP BY category ORDER BY count DESC, category ASC",
        category = request_error_category_expr()
    )
}

pub(super) fn error_trend_sql(granularity: SnapshotGranularity) -> String {
    format!(
        "SELECT {bucket} AS bucket_started_at, {category} AS category, COUNT(*)::bigint AS count \
        FROM request_records \
        WHERE created_at >= $1 AND created_at < $2 AND status IN ('failed', 'cancelled') \
        GROUP BY bucket_started_at, category ORDER BY bucket_started_at ASC, category ASC",
        bucket = bucket_expr("created_at", granularity),
        category = request_error_category_expr()
    )
}

pub(super) fn recent_errors_sql() -> &'static str {
    "SELECT created_at, request_id, provider_id, provider_name_snapshot AS provider_name, \
        COALESCE(model_name_snapshot, global_model_id) AS model, client_status_code AS status_code, \
        client_error_type AS error_type, client_error_message AS error_message, \
        total_latency_ms AS latency_ms, first_byte_time_ms AS ttfb_ms \
    FROM request_records \
    WHERE created_at >= $1 AND created_at < $2 AND status IN ('failed', 'cancelled') \
    ORDER BY created_at DESC, request_id DESC LIMIT $3"
}

pub(super) fn upstream_summary_sql(plan: &SnapshotQueryPlan, filters: &UpstreamFilters, slow_threshold_ms: i64) -> SqlParts {
    let mut query = upstream_base(plan, filters);
    let slow_index = query.push(slow_threshold_ms);
    query.sql.push_str(&format!(
        ") SELECT COUNT(*)::bigint AS request_count, \
            COALESCE(SUM(success_flag), 0)::bigint AS success_count, \
            COUNT(*)::bigint - COALESCE(SUM(success_flag), 0)::bigint AS error_count, \
            COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, \
            {tps} AS avg_output_tps, \
            AVG(success_ttfb_ms::double precision) AS avg_ttfb_ms, \
            AVG(success_latency_ms::double precision) AS avg_latency_ms, \
            percentile_disc(0.90) WITHIN GROUP (ORDER BY success_latency_ms) AS p90_latency_ms, \
            percentile_disc(0.99) WITHIN GROUP (ORDER BY success_latency_ms) AS p99_latency_ms, \
            percentile_disc(0.90) WITHIN GROUP (ORDER BY success_ttfb_ms) AS p90_ttfb_ms, \
            percentile_disc(0.99) WITHIN GROUP (ORDER BY success_ttfb_ms) AS p99_ttfb_ms, \
            COALESCE(SUM(CASE WHEN success_flag = 1 AND has_latency AND output_tokens > 0 THEN 1 ELSE 0 END), 0)::bigint AS tps_sample_count, \
            COUNT(success_latency_ms)::bigint AS latency_sample_count, \
            COUNT(success_ttfb_ms)::bigint AS ttfb_sample_count, \
            COALESCE(SUM(CASE WHEN has_latency AND latency_ms >= ${slow_index} THEN 1 ELSE 0 END), 0)::bigint AS slow_request_count \
        FROM shaped",
        tps = tps_expr(),
    ));
    query.finish()
}

pub(super) fn upstream_providers_sql(plan: &SnapshotQueryPlan, filters: &UpstreamFilters, limit: usize, slow_threshold_ms: i64) -> SqlParts {
    let mut query = upstream_base(plan, filters);
    let slow_index = query.push(slow_threshold_ms);
    let limit_index = query.push(i64::try_from(limit).unwrap_or(MAX_ANALYTICS_LIMIT as i64));
    query.sql.push_str(&format!(
        ") SELECT provider_id, provider_name, COUNT(*)::bigint AS request_count, \
            COALESCE(SUM(success_flag), 0)::bigint AS success_count, \
            COUNT(*)::bigint - COALESCE(SUM(success_flag), 0)::bigint AS error_count, \
            COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, \
            {tps} AS avg_output_tps, \
            AVG(success_ttfb_ms::double precision) AS avg_ttfb_ms, \
            AVG(success_latency_ms::double precision) AS avg_latency_ms, \
            percentile_disc(0.90) WITHIN GROUP (ORDER BY success_latency_ms) AS p90_latency_ms, \
            percentile_disc(0.99) WITHIN GROUP (ORDER BY success_latency_ms) AS p99_latency_ms, \
            percentile_disc(0.90) WITHIN GROUP (ORDER BY success_ttfb_ms) AS p90_ttfb_ms, \
            percentile_disc(0.99) WITHIN GROUP (ORDER BY success_ttfb_ms) AS p99_ttfb_ms, \
            COALESCE(SUM(CASE WHEN success_flag = 1 AND has_latency AND output_tokens > 0 THEN 1 ELSE 0 END), 0)::bigint AS tps_sample_count, \
            COUNT(success_latency_ms)::bigint AS latency_sample_count, \
            COUNT(success_ttfb_ms)::bigint AS ttfb_sample_count, \
            COALESCE(SUM(CASE WHEN has_latency AND latency_ms >= ${slow_index} THEN 1 ELSE 0 END), 0)::bigint AS slow_request_count \
        FROM shaped GROUP BY provider_id, provider_name ORDER BY request_count DESC, provider_name ASC LIMIT ${limit_index}",
        tps = tps_expr(),
    ));
    query.finish()
}

pub(super) fn upstream_timeline_sql(plan: &SnapshotQueryPlan, filters: &UpstreamFilters, provider_ids: &[String], slow_threshold_ms: i64) -> SqlParts {
    let mut query = upstream_base(plan, filters);
    let slow_index = query.push(slow_threshold_ms);
    let provider_indexes = provider_ids
        .iter()
        .map(|provider_id| format!("${}", query.push(provider_id.clone())))
        .collect::<Vec<_>>()
        .join(", ");
    let bucket = bucket_expr("created_at", plan.granularity);
    let bucket_end = bucket_end_expr("bucket_started_at", plan.granularity);
    query.sql.push_str(&format!(
        "), bucketed AS (SELECT {bucket} AS bucket_started_at, * FROM shaped WHERE provider_id IN ({provider_indexes})) \
        SELECT bucket_started_at, {bucket_end} AS bucket_ended_at, provider_id, provider_name, \
            COUNT(*)::bigint AS request_count, COALESCE(SUM(success_flag), 0)::bigint AS success_count, \
            COUNT(*)::bigint - COALESCE(SUM(success_flag), 0)::bigint AS error_count, COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, \
            {tps} AS avg_output_tps, AVG(success_ttfb_ms::double precision) AS avg_ttfb_ms, \
            AVG(success_latency_ms::double precision) AS avg_latency_ms, \
            COALESCE(SUM(CASE WHEN has_latency AND latency_ms >= ${slow_index} THEN 1 ELSE 0 END), 0)::bigint AS slow_request_count \
        FROM bucketed \
        GROUP BY bucket_started_at, provider_id, provider_name ORDER BY bucket_started_at ASC, provider_name ASC",
        tps = tps_expr(),
    ));
    query.finish()
}

fn upstream_base(plan: &SnapshotQueryPlan, filters: &UpstreamFilters) -> QueryBuilder {
    let mut query = QueryBuilder::new(
        "WITH filtered AS (SELECT * FROM request_candidates WHERE created_at >= $1 AND created_at < $2 AND started_at IS NOT NULL".into(),
        vec![Value::from(plan.started_at), Value::from(plan.ended_at)],
    );
    push_filter(&mut query, "provider_id = ", filters.provider_id.clone());
    push_model_filter(&mut query, filters.model.clone());
    push_filter(&mut query, "provider_api_format = ", filters.api_format.clone());
    push_bool_filter(&mut query, "is_stream = ", filters.is_stream);
    push_bool_filter(&mut query, "needs_conversion = ", filters.needs_conversion);
    query.sql.push_str("), shaped AS (");
    query.sql.push_str(upstream_shaped_sql());
    query
}

fn push_filter(query: &mut QueryBuilder, prefix: &str, value: Option<String>) {
    if let Some(value) = value {
        let index = query.push(value);
        query.sql.push_str(" AND ");
        query.sql.push_str(prefix);
        query.sql.push_str(&format!("${index}"));
    }
}

fn push_model_filter(query: &mut QueryBuilder, value: Option<String>) {
    if let Some(value) = value {
        let index = query.push(value);
        query.sql.push_str(" AND global_model_id = $");
        query.sql.push_str(&index.to_string());
    }
}

fn push_bool_filter(query: &mut QueryBuilder, prefix: &str, value: Option<bool>) {
    if let Some(value) = value {
        let index = query.push(value);
        query.sql.push_str(" AND ");
        query.sql.push_str(prefix);
        query.sql.push_str(&format!("${index}"));
    }
}

fn upstream_shaped_sql() -> &'static str {
    "SELECT created_at, COALESCE(provider_id, 'unknown') AS provider_id, \
        COALESCE(NULLIF(provider_name_snapshot, ''), provider_id, 'unknown') AS provider_name, \
        GREATEST(COALESCE(completion_tokens, output_text_tokens, total_tokens, 0), 0) AS output_tokens, \
        GREATEST(COALESCE(latency_ms, 0), 0) AS latency_ms, \
        GREATEST(COALESCE(first_byte_time_ms, 0), 0) AS first_byte_time_ms, \
        latency_ms IS NOT NULL AS has_latency, first_byte_time_ms IS NOT NULL AS has_ttfb, \
        CASE WHEN status = 'success' AND (status_code IS NULL OR status_code < 400) THEN 1 ELSE 0 END AS success_flag, \
        CASE WHEN status = 'success' AND (status_code IS NULL OR status_code < 400) AND latency_ms IS NOT NULL THEN GREATEST(latency_ms, 0) ELSE NULL END AS success_latency_ms, \
        CASE WHEN status = 'success' AND (status_code IS NULL OR status_code < 400) AND first_byte_time_ms IS NOT NULL THEN GREATEST(first_byte_time_ms, 0) ELSE NULL END AS success_ttfb_ms \
    FROM filtered"
}

fn tps_expr() -> &'static str {
    "CASE WHEN COALESCE(SUM(CASE WHEN success_flag = 1 AND has_latency AND output_tokens > 0 THEN latency_ms ELSE 0 END), 0) > 0 \
    THEN COALESCE(SUM(CASE WHEN success_flag = 1 AND has_latency AND output_tokens > 0 THEN output_tokens ELSE 0 END), 0)::double precision * 1000.0 / \
        COALESCE(SUM(CASE WHEN success_flag = 1 AND has_latency AND output_tokens > 0 THEN latency_ms ELSE 0 END), 0)::double precision \
    ELSE NULL END"
}

fn bucket_expr(column: &str, granularity: SnapshotGranularity) -> String {
    format!("date_trunc('{}', {column})", granularity.as_str())
}

fn bucket_end_expr(column: &str, granularity: SnapshotGranularity) -> String {
    format!("{column} + interval '{} seconds'", granularity.bucket_seconds())
}

fn request_error_category_expr() -> &'static str {
    "CASE \
        WHEN client_error_type IS NOT NULL THEN client_error_type \
        WHEN client_status_code >= 500 THEN 'server_error' \
        WHEN client_status_code = 429 THEN 'rate_limit' \
        WHEN client_status_code >= 400 THEN 'client_error' \
        WHEN termination_reason IS NOT NULL THEN termination_reason \
        ELSE 'unknown' END"
}

fn normalized_string(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

struct QueryBuilder {
    sql: String,
    values: Vec<Value>,
}

impl QueryBuilder {
    fn new(sql: String, values: Vec<Value>) -> Self {
        Self { sql, values }
    }

    fn push<T>(&mut self, value: T) -> usize
    where
        Value: From<T>,
    {
        self.values.push(Value::from(value));
        self.values.len()
    }

    fn finish(self) -> SqlParts {
        SqlParts {
            sql: self.sql,
            values: self.values,
        }
    }
}
