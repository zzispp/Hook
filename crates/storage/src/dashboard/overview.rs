use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement};
use types::dashboard::{DashboardBreakdownItem, DashboardBreakdowns, DashboardOverviewResponse, DashboardSummary, DashboardTimeseriesPoint, DashboardWindow};

use crate::{StorageError, StorageResult, database::Database};

use super::{
    DashboardBucketFilter, DashboardStore, DashboardStoreOverviewQuery,
    scope::{SqlParams, scope_response, scoped_time_where},
};

const BREAKDOWN_LIMIT: i64 = 8;

pub(super) async fn overview(store: &DashboardStore, query: DashboardStoreOverviewQuery) -> StorageResult<DashboardOverviewResponse> {
    let summary = summary(store, &query).await?;
    let timeseries = timeseries(store, &query).await?;
    let breakdowns = breakdowns(store, &query).await?;
    Ok(DashboardOverviewResponse {
        scope: scope_response(&query.scope),
        preset: query.preset,
        window: window_response(&query),
        summary,
        timeseries,
        breakdowns,
    })
}

async fn summary(store: &DashboardStore, query: &DashboardStoreOverviewQuery) -> StorageResult<DashboardSummary> {
    let mut params = SqlParams::new();
    let where_sql = scoped_time_where(&query.scope, query.started_at, query.ended_at, &mut params);
    let sql = format!("{} {}", summary_sql(), where_sql);
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values);
    let row = database(store)
        .connection()
        .query_one_raw(statement)
        .await?
        .ok_or_else(|| StorageError::Database("dashboard summary query returned no rows".into()))?;
    SummaryRow::from_query_result(&row, "").map(summary_response).map_err(StorageError::from)
}

async fn timeseries(store: &DashboardStore, query: &DashboardStoreOverviewQuery) -> StorageResult<Vec<DashboardTimeseriesPoint>> {
    let mut params = SqlParams::new();
    let offset = params.push(query.tz_offset_minutes);
    let where_sql = scoped_time_where(&query.scope, query.started_at, query.ended_at, &mut params);
    let sql = format!("{} {} {}", timeseries_select(query.bucket, &offset), where_sql, timeseries_group());
    let rows = TimeseriesRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await?;
    Ok(rows.into_iter().map(timeseries_response).collect())
}

async fn breakdowns(store: &DashboardStore, query: &DashboardStoreOverviewQuery) -> StorageResult<DashboardBreakdowns> {
    let models = breakdown_rows(
        store,
        query,
        "r.global_model_id",
        "COALESCE(r.model_name_snapshot, r.global_model_id, 'unknown')",
    )
    .await?;
    let api_formats = breakdown_rows(store, query, "r.client_api_format", "r.client_api_format").await?;
    let tokens = breakdown_rows(
        store,
        query,
        "r.token_id",
        "COALESCE(r.token_name_snapshot, r.token_prefix_snapshot, r.token_id, 'unknown')",
    )
    .await?;
    let providers = admin_breakdown(store, query, "r.provider_id", "COALESCE(r.provider_name_snapshot, r.provider_id, 'unknown')").await?;
    let users = admin_breakdown(
        store,
        query,
        "r.user_id_snapshot",
        "COALESCE(r.username_snapshot, r.user_id_snapshot, 'unknown')",
    )
    .await?;
    Ok(DashboardBreakdowns {
        models,
        api_formats,
        tokens,
        providers,
        users,
    })
}

async fn admin_breakdown(
    store: &DashboardStore,
    query: &DashboardStoreOverviewQuery,
    id_expression: &str,
    name_expression: &str,
) -> StorageResult<Vec<DashboardBreakdownItem>> {
    if !query.include_admin_breakdowns {
        return Ok(Vec::new());
    }
    breakdown_rows(store, query, id_expression, name_expression).await
}

async fn breakdown_rows(
    store: &DashboardStore,
    query: &DashboardStoreOverviewQuery,
    id_expression: &str,
    name_expression: &str,
) -> StorageResult<Vec<DashboardBreakdownItem>> {
    let mut params = SqlParams::new();
    let where_sql = scoped_time_where(&query.scope, query.started_at, query.ended_at, &mut params);
    let limit = params.push(BREAKDOWN_LIMIT);
    let sql = breakdown_sql(id_expression, name_expression, &where_sql, &limit);
    let rows = BreakdownRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await?;
    Ok(rows.into_iter().map(breakdown_response).collect())
}

fn summary_sql() -> &'static str {
    "SELECT \
        COUNT(*)::bigint AS request_count, \
        COUNT(*) FILTER (WHERE r.status = 'success')::bigint AS success_count, \
        COUNT(*) FILTER (WHERE r.status IN ('failed', 'cancelled'))::bigint AS failed_count, \
        COUNT(*) FILTER (WHERE r.status IN ('pending', 'streaming'))::bigint AS active_count, \
        COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
        AVG(r.total_latency_ms) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms, \
        AVG(r.first_byte_time_ms) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.first_byte_time_ms IS NOT NULL) AS avg_ttfb_ms, \
        COUNT(DISTINCT r.global_model_id) FILTER (WHERE r.global_model_id IS NOT NULL)::bigint AS model_count \
        FROM request_records r"
}

fn timeseries_select(bucket: DashboardBucketFilter, offset: &str) -> String {
    match bucket {
        DashboardBucketFilter::Hour => {
            format!(
                "SELECT to_char(date_trunc('hour', ((r.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD\"T\"HH24:MI:SS') AS bucket, \
            COUNT(*)::bigint AS request_count, \
            COUNT(*) FILTER (WHERE r.status = 'success')::bigint AS success_count, \
            COUNT(*) FILTER (WHERE r.status IN ('failed', 'cancelled'))::bigint AS failed_count, \
            COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
            COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
            AVG(r.total_latency_ms) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms \
            FROM request_records r"
            )
        }
        DashboardBucketFilter::Day => {
            format!(
                "SELECT to_char(date_trunc('day', ((r.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD') AS bucket, \
            COUNT(*)::bigint AS request_count, \
            COUNT(*) FILTER (WHERE r.status = 'success')::bigint AS success_count, \
            COUNT(*) FILTER (WHERE r.status IN ('failed', 'cancelled'))::bigint AS failed_count, \
            COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
            COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
            AVG(r.total_latency_ms) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL) AS avg_latency_ms \
            FROM request_records r"
            )
        }
    }
}

fn timeseries_group() -> &'static str {
    "GROUP BY bucket ORDER BY bucket ASC"
}

fn breakdown_sql(id_expression: &str, name_expression: &str, where_sql: &str, limit: &str) -> String {
    format!(
        "SELECT {id_expression} AS id, {name_expression} AS name, \
        COUNT(*)::bigint AS request_count, \
        COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost \
        FROM request_records r {where_sql} \
        GROUP BY id, name \
        ORDER BY request_count DESC, total_tokens DESC, name ASC \
        LIMIT {limit}"
    )
}

fn summary_response(row: SummaryRow) -> DashboardSummary {
    let success_count = row.success_count.unwrap_or_default();
    let failed_count = row.failed_count.unwrap_or_default();
    DashboardSummary {
        request_count: row.request_count.unwrap_or_default(),
        success_count,
        failed_count,
        active_count: row.active_count.unwrap_or_default(),
        success_rate: success_rate(success_count, failed_count),
        total_tokens: row.total_tokens.unwrap_or_default(),
        total_cost: row.total_cost.unwrap_or(Decimal::ZERO),
        avg_latency_ms: row.avg_latency_ms,
        avg_ttfb_ms: row.avg_ttfb_ms,
        model_count: row.model_count.unwrap_or_default(),
    }
}

fn timeseries_response(row: TimeseriesRow) -> DashboardTimeseriesPoint {
    DashboardTimeseriesPoint {
        bucket: row.bucket,
        request_count: row.request_count.unwrap_or_default(),
        success_count: row.success_count.unwrap_or_default(),
        failed_count: row.failed_count.unwrap_or_default(),
        total_tokens: row.total_tokens.unwrap_or_default(),
        total_cost: row.total_cost.unwrap_or(Decimal::ZERO),
        avg_latency_ms: row.avg_latency_ms,
    }
}

fn breakdown_response(row: BreakdownRow) -> DashboardBreakdownItem {
    DashboardBreakdownItem {
        id: row.id,
        name: row.name,
        request_count: row.request_count.unwrap_or_default(),
        total_tokens: row.total_tokens.unwrap_or_default(),
        total_cost: row.total_cost.unwrap_or(Decimal::ZERO),
    }
}

fn success_rate(success_count: i64, failed_count: i64) -> f64 {
    let denominator = success_count + failed_count;
    if denominator <= 0 {
        return 0.0;
    }
    success_count as f64 / denominator as f64
}

fn window_response(query: &DashboardStoreOverviewQuery) -> DashboardWindow {
    DashboardWindow {
        started_at: format_timestamp(query.started_at),
        ended_at: format_timestamp(query.ended_at),
        bucket: bucket_name(query.bucket).into(),
    }
}

fn bucket_name(bucket: DashboardBucketFilter) -> &'static str {
    match bucket {
        DashboardBucketFilter::Hour => "hour",
        DashboardBucketFilter::Day => "day",
    }
}

fn format_timestamp(value: time::OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .expect("dashboard timestamp must format as RFC3339")
}

fn database(store: &DashboardStore) -> &Database {
    store.database()
}

#[derive(Debug, FromQueryResult)]
struct SummaryRow {
    request_count: Option<i64>,
    success_count: Option<i64>,
    failed_count: Option<i64>,
    active_count: Option<i64>,
    total_tokens: Option<i64>,
    total_cost: Option<Decimal>,
    avg_latency_ms: Option<f64>,
    avg_ttfb_ms: Option<f64>,
    model_count: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct TimeseriesRow {
    bucket: String,
    request_count: Option<i64>,
    success_count: Option<i64>,
    failed_count: Option<i64>,
    total_tokens: Option<i64>,
    total_cost: Option<Decimal>,
    avg_latency_ms: Option<f64>,
}

#[derive(Debug, FromQueryResult)]
struct BreakdownRow {
    id: Option<String>,
    name: String,
    request_count: Option<i64>,
    total_tokens: Option<i64>,
    total_cost: Option<Decimal>,
}
