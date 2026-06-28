use super::{
    analytics_sql::MAX_ANALYTICS_LIMIT,
    analytics_upstream_query::{HistogramCteInput, QueryBuilder, SqlParts, UpstreamFilters, push_filters, push_provider_id_filter},
    types::SnapshotQueryPlan,
};

pub(super) fn upstream_summary_sql(plan: &SnapshotQueryPlan, filters: &UpstreamFilters, slow_threshold_ms: i64) -> SqlParts {
    let mut query = QueryBuilder::metric(plan);
    push_filters(&mut query, "b", filters);
    query.sql.push_str("), aggregated AS (");
    query.sql.push_str(&upstream_aggregate_sql());
    push_histogram_ctes(&mut query, filters, HistogramCteInput::empty());
    let slow_index = query.push(slow_threshold_ms);
    query.sql.push_str(&format!(
        ") SELECT aggregated.*, {p90_latency}, {p99_latency}, {p90_ttfb}, {p99_ttfb}, {p90_response_headers}, {p99_response_headers}, \
        {p90_first_sse_event}, {p99_first_sse_event}, {p90_first_output}, {p99_first_output}, {p90_sse_to_output}, {p99_sse_to_output}, {slow_count} FROM aggregated",
        p90_latency = percentile_expr("latency", "0.90", "p90_latency_ms"),
        p99_latency = percentile_expr("latency", "0.99", "p99_latency_ms"),
        p90_ttfb = percentile_expr("ttfb", "0.90", "p90_ttfb_ms"),
        p99_ttfb = percentile_expr("ttfb", "0.99", "p99_ttfb_ms"),
        p90_response_headers = percentile_expr("response_headers", "0.90", "p90_response_headers_ms"),
        p99_response_headers = percentile_expr("response_headers", "0.99", "p99_response_headers_ms"),
        p90_first_sse_event = percentile_expr("first_sse_event", "0.90", "p90_first_sse_event_ms"),
        p99_first_sse_event = percentile_expr("first_sse_event", "0.99", "p99_first_sse_event_ms"),
        p90_first_output = percentile_expr("first_output", "0.90", "p90_first_output_ms"),
        p99_first_output = percentile_expr("first_output", "0.99", "p99_first_output_ms"),
        p90_sse_to_output = percentile_expr("sse_to_output", "0.90", "p90_sse_to_output_ms"),
        p99_sse_to_output = percentile_expr("sse_to_output", "0.99", "p99_sse_to_output_ms"),
        slow_count = slow_count_expr(slow_index, "slow_request_count"),
    ));
    query.finish()
}

pub(super) fn upstream_providers_sql(plan: &SnapshotQueryPlan, filters: &UpstreamFilters, limit: usize, slow_threshold_ms: i64) -> SqlParts {
    let mut query = QueryBuilder::metric(plan);
    push_filters(&mut query, "b", filters);
    query.sql.push_str("), aggregated AS (");
    query.sql.push_str(&upstream_provider_aggregate_sql());
    push_histogram_ctes(&mut query, filters, HistogramCteInput::provider());
    let slow_index = query.push(slow_threshold_ms);
    let limit_index = query.push(i64::try_from(limit).unwrap_or(MAX_ANALYTICS_LIMIT as i64));
    query.sql.push_str(&format!(
        ") SELECT aggregated.*, {p90_latency}, {p99_latency}, {p90_ttfb}, {p99_ttfb}, {p90_response_headers}, {p99_response_headers}, \
        {p90_first_sse_event}, {p99_first_sse_event}, {p90_first_output}, {p99_first_output}, {p90_sse_to_output}, {p99_sse_to_output}, {slow_count} \
        FROM aggregated ORDER BY request_count DESC, provider_name ASC LIMIT ${limit_index}",
        p90_latency = provider_percentile_expr("latency", "0.90", "p90_latency_ms"),
        p99_latency = provider_percentile_expr("latency", "0.99", "p99_latency_ms"),
        p90_ttfb = provider_percentile_expr("ttfb", "0.90", "p90_ttfb_ms"),
        p99_ttfb = provider_percentile_expr("ttfb", "0.99", "p99_ttfb_ms"),
        p90_response_headers = provider_percentile_expr("response_headers", "0.90", "p90_response_headers_ms"),
        p99_response_headers = provider_percentile_expr("response_headers", "0.99", "p99_response_headers_ms"),
        p90_first_sse_event = provider_percentile_expr("first_sse_event", "0.90", "p90_first_sse_event_ms"),
        p99_first_sse_event = provider_percentile_expr("first_sse_event", "0.99", "p99_first_sse_event_ms"),
        p90_first_output = provider_percentile_expr("first_output", "0.90", "p90_first_output_ms"),
        p99_first_output = provider_percentile_expr("first_output", "0.99", "p99_first_output_ms"),
        p90_sse_to_output = provider_percentile_expr("sse_to_output", "0.90", "p90_sse_to_output_ms"),
        p99_sse_to_output = provider_percentile_expr("sse_to_output", "0.99", "p99_sse_to_output_ms"),
        slow_count = provider_slow_count_expr(slow_index, "slow_request_count"),
    ));
    query.finish()
}

pub(super) fn upstream_timeline_sql(plan: &SnapshotQueryPlan, filters: &UpstreamFilters, provider_ids: &[String], slow_threshold_ms: i64) -> SqlParts {
    let mut query = QueryBuilder::metric(plan);
    push_filters(&mut query, "b", filters);
    push_provider_id_filter(&mut query, "b", provider_ids);
    query.sql.push_str("), aggregated AS (");
    query.sql.push_str(&upstream_timeline_aggregate_sql());
    push_histogram_ctes(&mut query, filters, HistogramCteInput::timeline(provider_ids));
    let slow_index = query.push(slow_threshold_ms);
    query.sql.push_str(&format!(
        ") SELECT aggregated.*, {} FROM aggregated ORDER BY bucket_started_at ASC, provider_name ASC",
        timeline_slow_count_expr(slow_index, "slow_request_count")
    ));
    query.finish()
}

fn upstream_aggregate_sql() -> String {
    format!(
        "SELECT COALESCE(SUM(request_count), 0)::bigint AS request_count, COALESCE(SUM(success_count), 0)::bigint AS success_count, \
        COALESCE(SUM(failed_count), 0)::bigint AS error_count, COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, \
        {} AS avg_output_tps, {} AS avg_ttfb_ms, {} AS avg_latency_ms, {} AS avg_response_headers_ms, {} AS avg_first_sse_event_ms, \
        {} AS avg_first_output_ms, {} AS avg_sse_to_output_ms, COALESCE(SUM(tps_sample_count), 0)::bigint AS tps_sample_count, \
        COALESCE(SUM(latency_sample_count), 0)::bigint AS latency_sample_count, COALESCE(SUM(ttfb_sample_count), 0)::bigint AS ttfb_sample_count, \
        COALESCE(SUM(response_headers_sample_count), 0)::bigint AS response_headers_sample_count, \
        COALESCE(SUM(first_sse_event_sample_count), 0)::bigint AS first_sse_event_sample_count, \
        COALESCE(SUM(first_output_sample_count), 0)::bigint AS first_output_sample_count, \
        COALESCE(SUM(sse_to_output_sample_count), 0)::bigint AS sse_to_output_sample_count FROM filtered",
        tps_expr(),
        avg_expr("ttfb_total_ms", "ttfb_sample_count"),
        avg_expr("latency_total_ms", "latency_sample_count"),
        avg_expr("response_headers_total_ms", "response_headers_sample_count"),
        avg_expr("first_sse_event_total_ms", "first_sse_event_sample_count"),
        avg_expr("first_output_total_ms", "first_output_sample_count"),
        avg_expr("sse_to_output_total_ms", "sse_to_output_sample_count")
    )
}

fn upstream_provider_aggregate_sql() -> String {
    format!(
        "SELECT COALESCE(provider_id, 'unknown') AS provider_id, COALESCE(MAX(provider_name), COALESCE(provider_id, 'unknown')) AS provider_name, \
        COALESCE(SUM(request_count), 0)::bigint AS request_count, COALESCE(SUM(success_count), 0)::bigint AS success_count, \
        COALESCE(SUM(failed_count), 0)::bigint AS error_count, COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, \
        {} AS avg_output_tps, {} AS avg_ttfb_ms, {} AS avg_latency_ms, {} AS avg_response_headers_ms, {} AS avg_first_sse_event_ms, \
        {} AS avg_first_output_ms, {} AS avg_sse_to_output_ms, COALESCE(SUM(tps_sample_count), 0)::bigint AS tps_sample_count, \
        COALESCE(SUM(latency_sample_count), 0)::bigint AS latency_sample_count, COALESCE(SUM(ttfb_sample_count), 0)::bigint AS ttfb_sample_count, \
        COALESCE(SUM(response_headers_sample_count), 0)::bigint AS response_headers_sample_count, \
        COALESCE(SUM(first_sse_event_sample_count), 0)::bigint AS first_sse_event_sample_count, \
        COALESCE(SUM(first_output_sample_count), 0)::bigint AS first_output_sample_count, \
        COALESCE(SUM(sse_to_output_sample_count), 0)::bigint AS sse_to_output_sample_count \
        FROM filtered GROUP BY provider_id",
        tps_expr(),
        avg_expr("ttfb_total_ms", "ttfb_sample_count"),
        avg_expr("latency_total_ms", "latency_sample_count"),
        avg_expr("response_headers_total_ms", "response_headers_sample_count"),
        avg_expr("first_sse_event_total_ms", "first_sse_event_sample_count"),
        avg_expr("first_output_total_ms", "first_output_sample_count"),
        avg_expr("sse_to_output_total_ms", "sse_to_output_sample_count")
    )
}

fn upstream_timeline_aggregate_sql() -> String {
    format!(
        "SELECT bucket_started_at, bucket_ended_at, COALESCE(provider_id, 'unknown') AS provider_id, COALESCE(MAX(provider_name), COALESCE(provider_id, 'unknown')) AS provider_name, \
        COALESCE(SUM(request_count), 0)::bigint AS request_count, COALESCE(SUM(success_count), 0)::bigint AS success_count, \
        COALESCE(SUM(failed_count), 0)::bigint AS error_count, COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, \
        {} AS avg_output_tps, {} AS avg_ttfb_ms, {} AS avg_latency_ms, {} AS avg_response_headers_ms, {} AS avg_first_sse_event_ms, \
        {} AS avg_first_output_ms, {} AS avg_sse_to_output_ms \
        FROM filtered GROUP BY bucket_started_at, bucket_ended_at, provider_id",
        tps_expr(),
        avg_expr("ttfb_total_ms", "ttfb_sample_count"),
        avg_expr("latency_total_ms", "latency_sample_count"),
        avg_expr("response_headers_total_ms", "response_headers_sample_count"),
        avg_expr("first_sse_event_total_ms", "first_sse_event_sample_count"),
        avg_expr("first_output_total_ms", "first_output_sample_count"),
        avg_expr("sse_to_output_total_ms", "sse_to_output_sample_count")
    )
}

fn tps_expr() -> &'static str {
    "CASE WHEN COALESCE(SUM(tps_latency_total_ms), 0) > 0 \
    THEN COALESCE(SUM(tps_output_tokens), 0)::double precision * 1000.0 / COALESCE(SUM(tps_latency_total_ms), 0)::double precision \
    ELSE NULL END"
}

fn avg_expr(total_column: &str, count_column: &str) -> String {
    format!("COALESCE(SUM({total_column}), 0)::double precision / NULLIF(COALESCE(SUM({count_column}), 0), 0)::double precision")
}

fn push_histogram_ctes(query: &mut QueryBuilder, filters: &UpstreamFilters, input: HistogramCteInput<'_>) {
    query.sql.push_str("), histogram_raw AS (SELECT ");
    query.sql.push_str(input.raw_select);
    query
        .sql
        .push_str("h.metric_kind, h.le_ms, COALESCE(SUM(h.sample_count), 0)::bigint AS bucket_count FROM dashboard_latency_histogram_buckets h ");
    query
        .sql
        .push_str("WHERE h.source_type = 'candidate' AND h.bucket_granularity = $3 AND h.bucket_started_at >= $1 AND h.bucket_started_at < $2");
    push_filters(query, "h", filters);
    push_provider_id_filter(query, "h", input.provider_ids);
    query.sql.push_str(" GROUP BY ");
    query.sql.push_str(input.raw_group);
    query
        .sql
        .push_str("h.metric_kind, h.le_ms), histogram AS (SELECT *, SUM(bucket_count) OVER (PARTITION BY ");
    query.sql.push_str(input.partition);
    query
        .sql
        .push_str("metric_kind ORDER BY le_ms)::bigint AS cumulative_count FROM histogram_raw), totals AS (SELECT ");
    query.sql.push_str(input.total_select);
    query
        .sql
        .push_str("metric_kind, SUM(bucket_count)::bigint AS total_count FROM histogram_raw GROUP BY ");
    query.sql.push_str(input.total_group);
    query.sql.push_str("metric_kind");
}

fn percentile_expr(metric_kind: &str, quantile: &str, alias: &str) -> String {
    format!(
        "(SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.metric_kind = h.metric_kind WHERE h.metric_kind = '{metric_kind}' AND h.cumulative_count >= CEIL(t.total_count::numeric * {quantile})) AS {alias}"
    )
}

fn provider_percentile_expr(metric_kind: &str, quantile: &str, alias: &str) -> String {
    format!(
        "(SELECT MIN(h.le_ms) FROM histogram h JOIN totals t ON t.provider_id = h.provider_id AND t.metric_kind = h.metric_kind WHERE h.provider_id = aggregated.provider_id AND h.metric_kind = '{metric_kind}' AND h.cumulative_count >= CEIL(t.total_count::numeric * {quantile})) AS {alias}"
    )
}

fn slow_count_expr(threshold_index: usize, alias: &str) -> String {
    format!(
        "(SELECT GREATEST(COALESCE(MAX(t.total_count), 0) - COALESCE(MAX(h.cumulative_count) FILTER (WHERE h.le_ms < ${threshold_index}), 0), 0)::bigint FROM totals t LEFT JOIN histogram h ON h.metric_kind = t.metric_kind WHERE t.metric_kind = 'latency') AS {alias}"
    )
}

fn provider_slow_count_expr(threshold_index: usize, alias: &str) -> String {
    format!(
        "(SELECT GREATEST(COALESCE(MAX(t.total_count), 0) - COALESCE(MAX(h.cumulative_count) FILTER (WHERE h.le_ms < ${threshold_index}), 0), 0)::bigint FROM totals t LEFT JOIN histogram h ON h.provider_id = t.provider_id AND h.metric_kind = t.metric_kind WHERE t.provider_id = aggregated.provider_id AND t.metric_kind = 'latency') AS {alias}"
    )
}

fn timeline_slow_count_expr(threshold_index: usize, alias: &str) -> String {
    format!(
        "(SELECT GREATEST(COALESCE(MAX(t.total_count), 0) - COALESCE(MAX(h.cumulative_count) FILTER (WHERE h.le_ms < ${threshold_index}), 0), 0)::bigint FROM totals t LEFT JOIN histogram h ON h.bucket_started_at = t.bucket_started_at AND h.provider_id = t.provider_id AND h.metric_kind = t.metric_kind WHERE t.bucket_started_at = aggregated.bucket_started_at AND t.provider_id = aggregated.provider_id AND t.metric_kind = 'latency') AS {alias}"
    )
}
