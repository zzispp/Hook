use super::DashboardBucketFilter;

pub(super) const BREAKDOWN_LIMIT: i64 = 8;

pub(super) fn summary_sql() -> &'static str {
    "SELECT \
        COUNT(*)::bigint AS request_count, \
        COUNT(*) FILTER (WHERE r.status = 'success')::bigint AS success_count, \
        COUNT(*) FILTER (WHERE r.status IN ('failed', 'cancelled'))::bigint AS failed_count, \
        COUNT(*) FILTER (WHERE r.status IN ('pending', 'streaming'))::bigint AS active_count, \
        COALESCE(SUM(COALESCE(r.prompt_tokens, 0)), 0)::bigint AS prompt_tokens, \
        COALESCE(SUM(COALESCE(r.cache_read_input_tokens, 0)), 0)::bigint AS cache_read_input_tokens, \
        COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
        COALESCE(SUM(COALESCE(r.upstream_total_cost, 0)), 0) AS upstream_total_cost, \
        AVG(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms, \
        AVG(r.first_byte_time_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.first_byte_time_ms IS NOT NULL) AS avg_ttfb_ms, \
        COUNT(DISTINCT r.global_model_id) FILTER (WHERE r.global_model_id IS NOT NULL)::bigint AS model_count \
        FROM request_records r"
}

pub(super) fn timeseries_select(bucket: DashboardBucketFilter, offset: &str) -> String {
    let time_expression = match bucket {
        DashboardBucketFilter::Hour => format!(
            "to_char(date_trunc('hour', ((r.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD\"T\"HH24:MI:SS')"
        ),
        DashboardBucketFilter::Day => format!(
            "to_char(date_trunc('day', ((r.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD')"
        ),
    };
    format!("SELECT {time_expression} AS bucket, {} FROM request_records r", timeseries_columns())
}

pub(super) fn timeseries_group() -> &'static str {
    "GROUP BY bucket ORDER BY bucket ASC"
}

pub(super) fn breakdown_sql(id_expression: &str, name_expression: &str, where_sql: &str, limit: &str) -> String {
    format!(
        "SELECT {id_expression} AS id, {name_expression} AS name, \
        COUNT(*)::bigint AS request_count, \
        COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
        COALESCE(SUM(COALESCE(r.upstream_total_cost, 0)), 0) AS upstream_total_cost, \
        AVG(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms \
        FROM request_records r {where_sql} \
        GROUP BY id, name \
        ORDER BY request_count DESC, total_tokens DESC, name ASC \
        LIMIT {limit}"
    )
}

fn timeseries_columns() -> &'static str {
    "COUNT(*)::bigint AS request_count, \
    COUNT(*) FILTER (WHERE r.status = 'success')::bigint AS success_count, \
    COUNT(*) FILTER (WHERE r.status IN ('failed', 'cancelled'))::bigint AS failed_count, \
    COALESCE(SUM(COALESCE(r.prompt_tokens, 0)), 0)::bigint AS prompt_tokens, \
    COALESCE(SUM(COALESCE(r.cache_read_input_tokens, 0)), 0)::bigint AS cache_read_input_tokens, \
    COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
    COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
    COALESCE(SUM(COALESCE(r.upstream_total_cost, 0)), 0) AS upstream_total_cost, \
    AVG(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms, \
    AVG(r.first_byte_time_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.first_byte_time_ms IS NOT NULL) AS avg_ttfb_ms"
}
