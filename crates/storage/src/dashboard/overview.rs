use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement};
use types::dashboard::{DashboardBreakdownItem, DashboardBreakdowns, DashboardOverviewResponse, DashboardSummary, DashboardTimeseriesPoint, DashboardWindow};

use crate::{StorageError, StorageResult};

use super::{
    DashboardBucketFilter, DashboardStore, DashboardStoreOverviewQuery,
    daily::daily_stats,
    money::admin_cost_metrics,
    overview_sql::{BREAKDOWN_LIMIT, SUMMARY_GRANULARITY, TIMESERIES_GRANULARITY, breakdown_sql, summary_sql, timeseries_group, timeseries_select},
    scope::{SqlParams, scope_response, scoped_metric_bucket_where},
};

pub(super) async fn overview(store: &DashboardStore, query: DashboardStoreOverviewQuery) -> StorageResult<DashboardOverviewResponse> {
    let summary = load_summary(store, &query, query.started_at, query.ended_at).await?;
    let today = load_summary(store, &query, query.today_started_at, query.today_ended_at).await?;
    let monthly = load_summary(store, &query, query.monthly_started_at, query.monthly_ended_at).await?;
    let timeseries = timeseries(store, &query).await?;
    let daily = daily_stats(store, &query).await?;
    let breakdowns = breakdowns(store, &query).await?;
    Ok(DashboardOverviewResponse {
        scope: scope_response(&query.scope),
        preset: query.preset,
        window: window_response(&query),
        summary,
        today,
        monthly,
        timeseries,
        daily,
        breakdowns,
    })
}

async fn load_summary(
    store: &DashboardStore,
    query: &DashboardStoreOverviewQuery,
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
) -> StorageResult<DashboardSummary> {
    let mut params = SqlParams::new();
    let where_sql = scoped_metric_bucket_where(&query.scope, started_at, ended_at, SUMMARY_GRANULARITY, &mut params);
    let sql = format!("{} {}", summary_sql(), where_sql);
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values);
    let row = store
        .database()
        .connection()
        .query_one_raw(statement)
        .await?
        .ok_or_else(|| StorageError::Database("dashboard summary query returned no rows".into()))?;
    SummaryRow::from_query_result(&row, "")
        .map(|row| summary_response(row, query.include_admin_costs))
        .map_err(StorageError::from)
}

async fn timeseries(store: &DashboardStore, query: &DashboardStoreOverviewQuery) -> StorageResult<Vec<DashboardTimeseriesPoint>> {
    let mut params = SqlParams::new();
    let offset = params.push(query.tz_offset_minutes);
    let where_sql = scoped_metric_bucket_where(&query.scope, query.started_at, query.ended_at, TIMESERIES_GRANULARITY, &mut params);
    let sql = format!("{} {} {}", timeseries_select(query.bucket, &offset), where_sql, timeseries_group());
    let rows = TimeseriesRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await?;
    Ok(rows.into_iter().map(|row| timeseries_response(row, query.include_admin_costs)).collect())
}

async fn breakdowns(store: &DashboardStore, query: &DashboardStoreOverviewQuery) -> StorageResult<DashboardBreakdowns> {
    let models = breakdown_rows(store, query, "b.global_model_id", "COALESCE(b.model_name, b.global_model_id, 'unknown')").await?;
    let api_formats = breakdown_rows(store, query, "b.client_api_format", "b.client_api_format").await?;
    let tokens = breakdown_rows(store, query, "b.token_id", "COALESCE(b.token_name, b.token_prefix, b.token_id, 'unknown')").await?;
    let providers = admin_breakdown(store, query, "b.provider_id", "COALESCE(b.provider_name, b.provider_id, 'unknown')").await?;
    let users = admin_breakdown(store, query, "b.user_id", "COALESCE(b.username, b.user_id, 'unknown')").await?;
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
    let where_sql = scoped_metric_bucket_where(&query.scope, query.started_at, query.ended_at, SUMMARY_GRANULARITY, &mut params);
    let limit = params.push(BREAKDOWN_LIMIT);
    let sql = breakdown_sql(id_expression, name_expression, &where_sql, &limit);
    let rows = BreakdownRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await?;
    Ok(rows.into_iter().map(|row| breakdown_response(row, query.include_admin_costs)).collect())
}

pub(super) fn summary_response(row: SummaryRow, include_admin_costs: bool) -> DashboardSummary {
    let success_count = row.success_count.unwrap_or_default();
    let failed_count = row.failed_count.unwrap_or_default();
    let prompt_tokens = row.prompt_tokens.unwrap_or_default();
    let cache_creation_tokens = row.cache_creation_input_tokens.unwrap_or_default();
    let cache_read_tokens = row.cache_read_input_tokens.unwrap_or_default();
    let total_cost = row.total_cost.unwrap_or(Decimal::ZERO);
    let metrics = admin_cost_metrics(total_cost, row.upstream_total_cost.unwrap_or(Decimal::ZERO), include_admin_costs);
    DashboardSummary {
        request_count: row.request_count.unwrap_or_default(),
        success_count,
        failed_count,
        active_count: row.active_count.unwrap_or_default(),
        success_rate: success_rate(success_count, failed_count),
        error_rate: error_rate(success_count, failed_count),
        cache_hit_rate: cache_hit_rate(cache_read_tokens, prompt_tokens, cache_creation_tokens),
        prompt_tokens,
        completion_tokens: row.completion_tokens.unwrap_or_default(),
        cache_creation_input_tokens: cache_creation_tokens,
        cache_read_input_tokens: cache_read_tokens,
        total_tokens: row.total_tokens.unwrap_or_default(),
        cache_creation_cost: row.cache_creation_cost.unwrap_or(Decimal::ZERO),
        cache_read_cost: row.cache_read_cost.unwrap_or(Decimal::ZERO),
        total_cost,
        upstream_total_cost: metrics.upstream_total_cost,
        profit: metrics.profit,
        profit_rate: metrics.profit_rate,
        avg_latency_ms: row.avg_latency_ms,
        avg_ttfb_ms: row.avg_ttfb_ms,
        avg_response_headers_ms: row.avg_response_headers_ms,
        avg_first_sse_event_ms: row.avg_first_sse_event_ms,
        avg_first_output_ms: row.avg_first_output_ms,
        avg_sse_to_output_ms: row.avg_sse_to_output_ms,
        model_count: row.model_count.unwrap_or_default(),
        provider_count: row.provider_count.unwrap_or_default(),
        user_count: row.user_count.unwrap_or_default(),
        token_count: row.token_count.unwrap_or_default(),
        failover_count: row.failover_count.unwrap_or_default(),
    }
}

pub(super) fn timeseries_response(row: TimeseriesRow, include_admin_costs: bool) -> DashboardTimeseriesPoint {
    let total_cost = row.total_cost.unwrap_or(Decimal::ZERO);
    let metrics = admin_cost_metrics(total_cost, row.upstream_total_cost.unwrap_or(Decimal::ZERO), include_admin_costs);
    DashboardTimeseriesPoint {
        bucket: row.bucket,
        request_count: row.request_count.unwrap_or_default(),
        success_count: row.success_count.unwrap_or_default(),
        failed_count: row.failed_count.unwrap_or_default(),
        total_tokens: row.total_tokens.unwrap_or_default(),
        total_cost,
        upstream_total_cost: metrics.upstream_total_cost,
        profit: metrics.profit,
        profit_rate: metrics.profit_rate,
        avg_latency_ms: row.avg_latency_ms,
        avg_ttfb_ms: row.avg_ttfb_ms,
        avg_response_headers_ms: row.avg_response_headers_ms,
        avg_first_sse_event_ms: row.avg_first_sse_event_ms,
        avg_first_output_ms: row.avg_first_output_ms,
        avg_sse_to_output_ms: row.avg_sse_to_output_ms,
        cache_hit_rate: cache_hit_rate(
            row.cache_read_input_tokens.unwrap_or_default(),
            row.prompt_tokens.unwrap_or_default(),
            row.cache_creation_input_tokens.unwrap_or_default(),
        ),
    }
}

pub(super) fn breakdown_response(row: BreakdownRow, include_admin_costs: bool) -> DashboardBreakdownItem {
    let total_cost = row.total_cost.unwrap_or(Decimal::ZERO);
    let metrics = admin_cost_metrics(total_cost, row.upstream_total_cost.unwrap_or(Decimal::ZERO), include_admin_costs);
    DashboardBreakdownItem {
        id: row.id,
        name: row.name,
        request_count: row.request_count.unwrap_or_default(),
        total_tokens: row.total_tokens.unwrap_or_default(),
        total_cost,
        upstream_total_cost: metrics.upstream_total_cost,
        profit: metrics.profit,
        profit_rate: metrics.profit_rate,
        avg_latency_ms: row.avg_latency_ms,
        avg_response_headers_ms: row.avg_response_headers_ms,
        avg_first_sse_event_ms: row.avg_first_sse_event_ms,
        avg_first_output_ms: row.avg_first_output_ms,
        avg_sse_to_output_ms: row.avg_sse_to_output_ms,
    }
}

fn success_rate(success_count: i64, failed_count: i64) -> f64 {
    let denominator = success_count + failed_count;
    if denominator <= 0 {
        return 0.0;
    }
    success_count as f64 / denominator as f64
}

fn error_rate(success_count: i64, failed_count: i64) -> f64 {
    let denominator = success_count + failed_count;
    if denominator <= 0 {
        return 0.0;
    }
    failed_count as f64 / denominator as f64
}

fn cache_hit_rate(cache_read_tokens: i64, prompt_tokens: i64, cache_creation_tokens: i64) -> f64 {
    let denominator = prompt_tokens + cache_creation_tokens + cache_read_tokens;
    if denominator <= 0 {
        return 0.0;
    }
    cache_read_tokens as f64 / denominator as f64
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

#[derive(Debug, FromQueryResult)]
pub(super) struct SummaryRow {
    pub(super) request_count: Option<i64>,
    pub(super) success_count: Option<i64>,
    pub(super) failed_count: Option<i64>,
    pub(super) active_count: Option<i64>,
    pub(super) prompt_tokens: Option<i64>,
    pub(super) completion_tokens: Option<i64>,
    pub(super) cache_creation_input_tokens: Option<i64>,
    pub(super) cache_read_input_tokens: Option<i64>,
    pub(super) total_tokens: Option<i64>,
    pub(super) cache_creation_cost: Option<Decimal>,
    pub(super) cache_read_cost: Option<Decimal>,
    pub(super) total_cost: Option<Decimal>,
    pub(super) upstream_total_cost: Option<Decimal>,
    pub(super) avg_latency_ms: Option<f64>,
    pub(super) avg_ttfb_ms: Option<f64>,
    pub(super) avg_response_headers_ms: Option<f64>,
    pub(super) avg_first_sse_event_ms: Option<f64>,
    pub(super) avg_first_output_ms: Option<f64>,
    pub(super) avg_sse_to_output_ms: Option<f64>,
    pub(super) model_count: Option<i64>,
    pub(super) provider_count: Option<i64>,
    pub(super) user_count: Option<i64>,
    pub(super) token_count: Option<i64>,
    pub(super) failover_count: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct TimeseriesRow {
    pub(super) bucket: String,
    pub(super) request_count: Option<i64>,
    pub(super) success_count: Option<i64>,
    pub(super) failed_count: Option<i64>,
    pub(super) prompt_tokens: Option<i64>,
    pub(super) cache_creation_input_tokens: Option<i64>,
    pub(super) cache_read_input_tokens: Option<i64>,
    pub(super) total_tokens: Option<i64>,
    pub(super) total_cost: Option<Decimal>,
    pub(super) upstream_total_cost: Option<Decimal>,
    pub(super) avg_latency_ms: Option<f64>,
    pub(super) avg_ttfb_ms: Option<f64>,
    pub(super) avg_response_headers_ms: Option<f64>,
    pub(super) avg_first_sse_event_ms: Option<f64>,
    pub(super) avg_first_output_ms: Option<f64>,
    pub(super) avg_sse_to_output_ms: Option<f64>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct BreakdownRow {
    pub(super) id: Option<String>,
    pub(super) name: String,
    pub(super) request_count: Option<i64>,
    pub(super) total_tokens: Option<i64>,
    pub(super) total_cost: Option<Decimal>,
    pub(super) upstream_total_cost: Option<Decimal>,
    pub(super) avg_latency_ms: Option<f64>,
    pub(super) avg_response_headers_ms: Option<f64>,
    pub(super) avg_first_sse_event_ms: Option<f64>,
    pub(super) avg_first_output_ms: Option<f64>,
    pub(super) avg_sse_to_output_ms: Option<f64>,
}
