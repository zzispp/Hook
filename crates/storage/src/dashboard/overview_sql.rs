use super::DashboardBucketFilter;

pub(super) const BREAKDOWN_LIMIT: i64 = 8;
pub(super) const SUMMARY_GRANULARITY: &str = "minute";
pub(super) const TIMESERIES_GRANULARITY: &str = "hour";

pub(super) fn summary_sql() -> &'static str {
    "SELECT \
        COALESCE(SUM(b.request_count), 0)::bigint AS request_count, \
        COALESCE(SUM(b.success_count), 0)::bigint AS success_count, \
        COALESCE(SUM(b.failed_count), 0)::bigint AS failed_count, \
        COALESCE(SUM(b.active_count), 0)::bigint AS active_count, \
        COALESCE(SUM(b.prompt_tokens), 0)::bigint AS prompt_tokens, \
        COALESCE(SUM(b.completion_tokens), 0)::bigint AS completion_tokens, \
        COALESCE(SUM(b.cache_creation_input_tokens), 0)::bigint AS cache_creation_input_tokens, \
        COALESCE(SUM(b.cache_read_input_tokens), 0)::bigint AS cache_read_input_tokens, \
        COALESCE(SUM(b.total_tokens), 0)::bigint AS total_tokens, \
        COALESCE(SUM(b.cache_creation_cost), 0) AS cache_creation_cost, \
        COALESCE(SUM(b.cache_read_cost), 0) AS cache_read_cost, \
        COALESCE(SUM(b.total_cost), 0) AS total_cost, \
        COALESCE(SUM(b.upstream_total_cost), 0) AS upstream_total_cost, \
        COALESCE(SUM(b.latency_total_ms), 0)::double precision / NULLIF(COALESCE(SUM(b.latency_sample_count), 0), 0)::double precision AS avg_latency_ms, \
        COALESCE(SUM(b.ttfb_total_ms), 0)::double precision / NULLIF(COALESCE(SUM(b.ttfb_sample_count), 0), 0)::double precision AS avg_ttfb_ms, \
        COUNT(DISTINCT b.global_model_id) FILTER (WHERE b.global_model_id IS NOT NULL AND b.request_count <> 0)::bigint AS model_count, \
        COUNT(DISTINCT b.provider_id) FILTER (WHERE b.provider_id IS NOT NULL AND b.request_count <> 0)::bigint AS provider_count, \
        COUNT(DISTINCT b.user_id) FILTER (WHERE b.user_id IS NOT NULL AND b.request_count <> 0)::bigint AS user_count, \
        COUNT(DISTINCT b.token_id) FILTER (WHERE b.token_id IS NOT NULL AND b.request_count <> 0)::bigint AS token_count, \
        COALESCE(SUM(b.failover_count), 0)::bigint AS failover_count \
        FROM dashboard_request_metric_buckets b"
}

pub(super) fn timeseries_select(bucket: DashboardBucketFilter, offset: &str) -> String {
    let time_expression = match bucket {
        DashboardBucketFilter::Hour => {
            format!(
                "to_char(date_trunc('hour', ((b.bucket_started_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD\"T\"HH24:MI:SS')"
            )
        }
        DashboardBucketFilter::Day => {
            format!("to_char(date_trunc('day', ((b.bucket_started_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD')")
        }
    };
    format!(
        "SELECT {time_expression} AS bucket, {} FROM dashboard_request_metric_buckets b",
        timeseries_columns()
    )
}

pub(super) fn timeseries_group() -> &'static str {
    "GROUP BY 1 ORDER BY 1 ASC"
}

pub(super) fn breakdown_sql(id_expression: &str, name_expression: &str, where_sql: &str, limit: &str) -> String {
    format!(
        "SELECT {id_expression} AS id, {name_expression} AS name, \
        COALESCE(SUM(b.request_count), 0)::bigint AS request_count, \
        COALESCE(SUM(b.total_tokens), 0)::bigint AS total_tokens, \
        COALESCE(SUM(b.total_cost), 0) AS total_cost, \
        COALESCE(SUM(b.upstream_total_cost), 0) AS upstream_total_cost, \
        COALESCE(SUM(b.latency_total_ms), 0)::double precision / NULLIF(COALESCE(SUM(b.latency_sample_count), 0), 0)::double precision AS avg_latency_ms \
        FROM dashboard_request_metric_buckets b {where_sql} \
        GROUP BY 1, 2 \
        ORDER BY request_count DESC, total_tokens DESC, name ASC \
        LIMIT {limit}"
    )
}

fn timeseries_columns() -> &'static str {
    "COALESCE(SUM(b.request_count), 0)::bigint AS request_count, \
    COALESCE(SUM(b.success_count), 0)::bigint AS success_count, \
    COALESCE(SUM(b.failed_count), 0)::bigint AS failed_count, \
    COALESCE(SUM(b.prompt_tokens), 0)::bigint AS prompt_tokens, \
    COALESCE(SUM(b.cache_creation_input_tokens), 0)::bigint AS cache_creation_input_tokens, \
    COALESCE(SUM(b.cache_read_input_tokens), 0)::bigint AS cache_read_input_tokens, \
    COALESCE(SUM(b.total_tokens), 0)::bigint AS total_tokens, \
    COALESCE(SUM(b.total_cost), 0) AS total_cost, \
    COALESCE(SUM(b.upstream_total_cost), 0) AS upstream_total_cost, \
    COALESCE(SUM(b.latency_total_ms), 0)::double precision / NULLIF(COALESCE(SUM(b.latency_sample_count), 0), 0)::double precision AS avg_latency_ms, \
    COALESCE(SUM(b.ttfb_total_ms), 0)::double precision / NULLIF(COALESCE(SUM(b.ttfb_sample_count), 0), 0)::double precision AS avg_ttfb_ms"
}
