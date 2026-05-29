use super::DashboardBucketFilter;
use super::token_context::{sum_cache_creation_tokens_sql, sum_cache_read_tokens_sql, sum_total_tokens_sql};

pub(super) const BREAKDOWN_LIMIT: i64 = 8;

pub(super) fn summary_sql() -> String {
    format!(
        "SELECT \
        COUNT(*)::bigint AS request_count, \
        COUNT(*) FILTER (WHERE r.status = 'success')::bigint AS success_count, \
        COUNT(*) FILTER (WHERE r.status IN ('failed', 'cancelled'))::bigint AS failed_count, \
        COUNT(*) FILTER (WHERE r.status IN ('pending', 'streaming'))::bigint AS active_count, \
        COALESCE(SUM(COALESCE(r.prompt_tokens, 0)), 0)::bigint AS prompt_tokens, \
        COALESCE(SUM(COALESCE(r.completion_tokens, 0)), 0)::bigint AS completion_tokens, \
        {} AS cache_creation_input_tokens, \
        {} AS cache_read_input_tokens, \
        {} AS total_tokens, \
        COALESCE(SUM(COALESCE(r.cache_creation_cost, 0)), 0) AS cache_creation_cost, \
        COALESCE(SUM(COALESCE(r.cache_read_cost, 0)), 0) AS cache_read_cost, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
        COALESCE(SUM(COALESCE(r.upstream_total_cost, 0)), 0) AS upstream_total_cost, \
        AVG(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms, \
        AVG(r.first_byte_time_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.first_byte_time_ms IS NOT NULL) AS avg_ttfb_ms, \
        COUNT(DISTINCT r.global_model_id) FILTER (WHERE r.global_model_id IS NOT NULL)::bigint AS model_count, \
        COUNT(DISTINCT r.provider_id) FILTER (WHERE r.provider_id IS NOT NULL)::bigint AS provider_count, \
        COUNT(DISTINCT r.user_id_snapshot) FILTER (WHERE r.user_id_snapshot IS NOT NULL)::bigint AS user_count, \
        COUNT(DISTINCT r.token_id) FILTER (WHERE r.token_id IS NOT NULL)::bigint AS token_count, \
        COUNT(*) FILTER (WHERE r.has_failover)::bigint AS failover_count \
        FROM request_records r",
        sum_cache_creation_tokens_sql("r"),
        sum_cache_read_tokens_sql("r"),
        sum_total_tokens_sql("r")
    )
}

pub(super) fn timeseries_select(bucket: DashboardBucketFilter, offset: &str) -> String {
    let time_expression = match bucket {
        DashboardBucketFilter::Hour => {
            format!("to_char(date_trunc('hour', ((r.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD\"T\"HH24:MI:SS')")
        }
        DashboardBucketFilter::Day => {
            format!("to_char(date_trunc('day', ((r.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD')")
        }
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
        {} AS total_tokens, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
        COALESCE(SUM(COALESCE(r.upstream_total_cost, 0)), 0) AS upstream_total_cost, \
        AVG(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms \
        FROM request_records r {where_sql} \
        GROUP BY id, name \
        ORDER BY request_count DESC, total_tokens DESC, name ASC \
        LIMIT {limit}",
        sum_total_tokens_sql("r")
    )
}

fn timeseries_columns() -> String {
    format!(
        "COUNT(*)::bigint AS request_count, \
    COUNT(*) FILTER (WHERE r.status = 'success')::bigint AS success_count, \
    COUNT(*) FILTER (WHERE r.status IN ('failed', 'cancelled'))::bigint AS failed_count, \
    COALESCE(SUM(COALESCE(r.prompt_tokens, 0)), 0)::bigint AS prompt_tokens, \
    {} AS cache_creation_input_tokens, \
    {} AS cache_read_input_tokens, \
    {} AS total_tokens, \
    COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
    COALESCE(SUM(COALESCE(r.upstream_total_cost, 0)), 0) AS upstream_total_cost, \
    AVG(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms, \
    AVG(r.first_byte_time_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.first_byte_time_ms IS NOT NULL) AS avg_ttfb_ms",
        sum_cache_creation_tokens_sql("r"),
        sum_cache_read_tokens_sql("r"),
        sum_total_tokens_sql("r")
    )
}
