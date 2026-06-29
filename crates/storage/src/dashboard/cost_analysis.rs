use rust_decimal::{Decimal, prelude::ToPrimitive};
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, TransactionTrait};
use types::dashboard::{
    DashboardApiKeyLeaderboardResponse, DashboardCostForecastPoint, DashboardCostForecastResponse, DashboardCostSavingsResponse,
    DashboardProviderAggregationItem, DashboardSortOrder, DashboardUserStatsLeaderboardItem, DashboardUserStatsMetric,
};

use crate::{StorageError, StorageResult, provider::record::request_records};

use super::{
    DashboardApiKeyLeaderboardQuery, DashboardCostForecastQuery, DashboardCostSavingsQuery, DashboardProviderAggregationQuery, DashboardStore,
    latency_stage::StageLatencyContribution, scope::SqlParams, token_context,
};

const DIMENSION_GLOBAL: &str = "global";
const DIMENSION_PROVIDER: &str = "provider";
const DIMENSION_API_KEY: &str = "api_key";
const GLOBAL_DIMENSION_ID: &str = "global";
const TERMINAL_SUCCESS: &str = "success";
const TERMINAL_FAILED: &str = "failed";
const TERMINAL_CANCELLED: &str = "cancelled";
const ADMIN_ROLE: &str = "admin";
const SHARD_COUNT: u64 = 32;
const CACHE_READ_ESTIMATE_MULTIPLIER: u32 = 10;
const PERCENT_MULTIPLIER: f64 = 100.0;
const COST_RESPONSE_SCALE: u32 = 6;
const FORECAST_RESPONSE_SCALE: u32 = 4;

#[derive(Clone, Copy, Debug)]
struct ApiKeyFilters {
    include_inactive: bool,
    exclude_admin: bool,
}

#[derive(Clone, Copy, Debug)]
struct DimensionRowsQuery<'a> {
    window: &'a super::DashboardCostAnalysisWindow,
    dimension_kind: &'a str,
    limit: u64,
    offset: u64,
    metric: DashboardUserStatsMetric,
    order: DashboardSortOrder,
    filters: ApiKeyFilters,
}

pub async fn sync_cost_analysis_buckets(
    connection: &sea_orm::DatabaseConnection,
    old_record: &request_records::Model,
    new_record: &request_records::Model,
) -> StorageResult<()> {
    let old_contribution = BucketContribution::from_record(old_record);
    let new_contribution = BucketContribution::from_record(new_record);
    if old_contribution.is_none() && new_contribution.is_none() {
        return Ok(());
    }
    let tx = connection.begin().await?;
    apply_delta(&tx, old_contribution.as_ref(), -1).await?;
    apply_delta(&tx, new_contribution.as_ref(), 1).await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn forecast(store: &DashboardStore, query: DashboardCostForecastQuery) -> StorageResult<DashboardCostForecastResponse> {
    let rows = daily_global_rows(store, &query.window).await?;
    let history = rows
        .into_iter()
        .map(|row| DashboardCostForecastPoint {
            date: row.date,
            total_cost: round_cost(row.total_cost.unwrap_or(Decimal::ZERO), COST_RESPONSE_SCALE),
        })
        .collect::<Vec<_>>();
    let values = history.iter().map(|point| point.total_cost.to_f64().unwrap_or(0.0)).collect::<Vec<_>>();
    let (slope, intercept) = linear_regression(&values);
    let forecast = forecast_points(
        history.last().map(|point| point.date.as_str()),
        query.window.end_date,
        query.forecast_days,
        &values,
        slope,
        intercept,
    )?;
    Ok(DashboardCostForecastResponse {
        history,
        forecast,
        slope: round_float(slope, COST_RESPONSE_SCALE),
        intercept: round_float(intercept, COST_RESPONSE_SCALE),
        start_date: query.window.start_date.to_string(),
        end_date: query.window.end_date.to_string(),
    })
}

pub(super) async fn savings(store: &DashboardStore, query: DashboardCostSavingsQuery) -> StorageResult<DashboardCostSavingsResponse> {
    let row = global_savings_row(store, &query.window).await?;
    let cache_read_cost = row.cache_read_cost.unwrap_or(Decimal::ZERO);
    let estimated_full_cost = effective_estimated_full_cost(row.estimated_full_cost.unwrap_or(Decimal::ZERO), cache_read_cost);
    Ok(DashboardCostSavingsResponse {
        cache_read_tokens: row.cache_read_tokens.unwrap_or_default(),
        cache_read_cost: round_cost(cache_read_cost, COST_RESPONSE_SCALE),
        cache_creation_cost: round_cost(row.cache_creation_cost.unwrap_or(Decimal::ZERO), COST_RESPONSE_SCALE),
        estimated_full_cost: round_cost(estimated_full_cost, COST_RESPONSE_SCALE),
        cache_savings: round_cost(estimated_full_cost - cache_read_cost, COST_RESPONSE_SCALE),
    })
}

pub(super) async fn api_key_leaderboard(store: &DashboardStore, query: DashboardApiKeyLeaderboardQuery) -> StorageResult<DashboardApiKeyLeaderboardResponse> {
    let filters = ApiKeyFilters {
        include_inactive: query.include_inactive,
        exclude_admin: query.exclude_admin,
    };
    let rows = dimension_rows(
        store,
        DimensionRowsQuery {
            window: &query.window,
            dimension_kind: DIMENSION_API_KEY,
            limit: query.limit,
            offset: query.offset,
            metric: query.metric,
            order: query.order,
            filters,
        },
    )
    .await?;
    let total = dimension_total(store, &query.window, DIMENSION_API_KEY, filters).await?;
    let items = rows
        .into_iter()
        .map(|row| {
            let value = metric_decimal(query.metric, &row);
            DashboardUserStatsLeaderboardItem {
                rank: row.rank.unwrap_or_default() as u64,
                id: row.dimension_id,
                name: row.dimension_name.unwrap_or(row.fallback_name),
                value: round_leaderboard_value(query.metric, value),
                requests: row.request_count.unwrap_or_default(),
                tokens: row.total_tokens.unwrap_or_default(),
                cost: round_cost(row.total_cost.unwrap_or(Decimal::ZERO), COST_RESPONSE_SCALE),
            }
        })
        .collect();
    Ok(DashboardApiKeyLeaderboardResponse {
        items,
        total,
        metric: query.metric,
        start_date: query.window.start_date.to_string(),
        end_date: query.window.end_date.to_string(),
    })
}

pub(super) async fn provider_aggregation(
    store: &DashboardStore,
    query: DashboardProviderAggregationQuery,
) -> StorageResult<Vec<DashboardProviderAggregationItem>> {
    let rows = provider_rows(store, &query).await?;
    Ok(rows.into_iter().map(provider_item).collect())
}

async fn apply_delta<C>(connection: &C, contribution: Option<&BucketContribution>, multiplier: i64) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let Some(contribution) = contribution else {
        return Ok(());
    };
    for dimension in contribution.dimensions() {
        upsert_bucket_delta(connection, contribution, &dimension, multiplier).await?;
    }
    Ok(())
}

async fn upsert_bucket_delta<C>(connection: &C, contribution: &BucketContribution, dimension: &Dimension, multiplier: i64) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let bounds = hour_bounds(contribution.created_at);
    let now = time::OffsetDateTime::now_utc();
    let shard = shard(&contribution.request_id);
    let mut params = SqlParams::new();
    let sql = format!(
        "INSERT INTO dashboard_cost_analysis_buckets \
        (id, bucket_started_at, bucket_ended_at, dimension_kind, dimension_id, dimension_name, shard, request_count, success_count, failed_count, \
        input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens, total_tokens, total_cost, upstream_total_cost, cache_read_cost, cache_creation_cost, \
        estimated_full_cost, total_latency_ms, latency_sample_count, response_headers_total_ms, response_headers_sample_count, first_byte_total_ms, \
        first_byte_sample_count, first_sse_event_total_ms, first_sse_event_sample_count, first_output_total_ms, first_output_sample_count, \
        sse_to_output_total_ms, sse_to_output_sample_count, created_at, updated_at) \
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) \
        ON CONFLICT (bucket_started_at, dimension_kind, dimension_id, shard) DO UPDATE SET \
        dimension_name = COALESCE(EXCLUDED.dimension_name, dashboard_cost_analysis_buckets.dimension_name), \
        request_count = dashboard_cost_analysis_buckets.request_count + EXCLUDED.request_count, \
        success_count = dashboard_cost_analysis_buckets.success_count + EXCLUDED.success_count, \
        failed_count = dashboard_cost_analysis_buckets.failed_count + EXCLUDED.failed_count, \
        input_tokens = dashboard_cost_analysis_buckets.input_tokens + EXCLUDED.input_tokens, \
        output_tokens = dashboard_cost_analysis_buckets.output_tokens + EXCLUDED.output_tokens, \
        cache_creation_tokens = dashboard_cost_analysis_buckets.cache_creation_tokens + EXCLUDED.cache_creation_tokens, \
        cache_read_tokens = dashboard_cost_analysis_buckets.cache_read_tokens + EXCLUDED.cache_read_tokens, \
        total_tokens = dashboard_cost_analysis_buckets.total_tokens + EXCLUDED.total_tokens, \
        total_cost = dashboard_cost_analysis_buckets.total_cost + EXCLUDED.total_cost, \
        upstream_total_cost = dashboard_cost_analysis_buckets.upstream_total_cost + EXCLUDED.upstream_total_cost, \
        cache_read_cost = dashboard_cost_analysis_buckets.cache_read_cost + EXCLUDED.cache_read_cost, \
        cache_creation_cost = dashboard_cost_analysis_buckets.cache_creation_cost + EXCLUDED.cache_creation_cost, \
        estimated_full_cost = dashboard_cost_analysis_buckets.estimated_full_cost + EXCLUDED.estimated_full_cost, \
        total_latency_ms = dashboard_cost_analysis_buckets.total_latency_ms + EXCLUDED.total_latency_ms, \
        latency_sample_count = dashboard_cost_analysis_buckets.latency_sample_count + EXCLUDED.latency_sample_count, \
        response_headers_total_ms = dashboard_cost_analysis_buckets.response_headers_total_ms + EXCLUDED.response_headers_total_ms, \
        response_headers_sample_count = dashboard_cost_analysis_buckets.response_headers_sample_count + EXCLUDED.response_headers_sample_count, \
        first_byte_total_ms = dashboard_cost_analysis_buckets.first_byte_total_ms + EXCLUDED.first_byte_total_ms, \
        first_byte_sample_count = dashboard_cost_analysis_buckets.first_byte_sample_count + EXCLUDED.first_byte_sample_count, \
        first_sse_event_total_ms = dashboard_cost_analysis_buckets.first_sse_event_total_ms + EXCLUDED.first_sse_event_total_ms, \
        first_sse_event_sample_count = dashboard_cost_analysis_buckets.first_sse_event_sample_count + EXCLUDED.first_sse_event_sample_count, \
        first_output_total_ms = dashboard_cost_analysis_buckets.first_output_total_ms + EXCLUDED.first_output_total_ms, \
        first_output_sample_count = dashboard_cost_analysis_buckets.first_output_sample_count + EXCLUDED.first_output_sample_count, \
        sse_to_output_total_ms = dashboard_cost_analysis_buckets.sse_to_output_total_ms + EXCLUDED.sse_to_output_total_ms, \
        sse_to_output_sample_count = dashboard_cost_analysis_buckets.sse_to_output_sample_count + EXCLUDED.sse_to_output_sample_count, \
        updated_at = EXCLUDED.updated_at",
        params.push(uuid::Uuid::now_v7().to_string()),
        params.push(bounds.started_at),
        params.push(bounds.ended_at),
        params.push(dimension.kind.clone()),
        params.push(dimension.id.clone()),
        params.push(dimension.name.clone()),
        params.push(i64::from(shard)),
        params.push(multiplier),
        params.push(contribution.success_count * multiplier),
        params.push(contribution.failed_count * multiplier),
        params.push(contribution.input_tokens * multiplier),
        params.push(contribution.output_tokens * multiplier),
        params.push(contribution.cache_creation_tokens * multiplier),
        params.push(contribution.cache_read_tokens * multiplier),
        params.push(contribution.total_tokens * multiplier),
        params.push(contribution.total_cost * Decimal::from(multiplier)),
        params.push(contribution.upstream_total_cost * Decimal::from(multiplier)),
        params.push(contribution.cache_read_cost * Decimal::from(multiplier)),
        params.push(contribution.cache_creation_cost * Decimal::from(multiplier)),
        params.push(contribution.estimated_full_cost * Decimal::from(multiplier)),
        params.push(contribution.total_latency_ms * multiplier),
        params.push(contribution.latency_sample_count * multiplier),
        params.push(contribution.response_headers_total_ms * multiplier),
        params.push(contribution.response_headers_sample_count * multiplier),
        params.push(contribution.first_byte_total_ms * multiplier),
        params.push(contribution.first_byte_sample_count * multiplier),
        params.push(contribution.first_sse_event_total_ms * multiplier),
        params.push(contribution.first_sse_event_sample_count * multiplier),
        params.push(contribution.first_output_total_ms * multiplier),
        params.push(contribution.first_output_sample_count * multiplier),
        params.push(contribution.sse_to_output_total_ms * multiplier),
        params.push(contribution.sse_to_output_sample_count * multiplier),
        params.push(now),
        params.push(now)
    );
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

async fn daily_global_rows(store: &DashboardStore, window: &super::DashboardCostAnalysisWindow) -> StorageResult<Vec<DailyCostRow>> {
    let mut params = SqlParams::new();
    let offset = params.push(window.tz_offset_minutes);
    let sql = format!(
        "SELECT to_char(((bucket_started_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))::date, 'YYYY-MM-DD') AS date, \
        COALESCE(SUM(total_cost), 0) AS total_cost \
        FROM dashboard_cost_analysis_buckets {} \
        GROUP BY date ORDER BY date ASC",
        base_dimension_where(window, DIMENSION_GLOBAL, &mut params)
    );
    DailyCostRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map_err(Into::into)
}

async fn global_savings_row(store: &DashboardStore, window: &super::DashboardCostAnalysisWindow) -> StorageResult<SavingsRow> {
    let mut params = SqlParams::new();
    let sql = format!(
        "SELECT COALESCE(SUM(cache_read_tokens), 0)::bigint AS cache_read_tokens, \
        COALESCE(SUM(cache_read_cost), 0) AS cache_read_cost, \
        COALESCE(SUM(cache_creation_cost), 0) AS cache_creation_cost, \
        COALESCE(SUM(estimated_full_cost), 0) AS estimated_full_cost \
        FROM dashboard_cost_analysis_buckets {}",
        base_dimension_where(window, DIMENSION_GLOBAL, &mut params)
    );
    SavingsRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .one(store.database().connection())
        .await?
        .ok_or_else(|| StorageError::Database("dashboard cost savings query returned no rows".into()))
}

async fn dimension_rows(store: &DashboardStore, query: DimensionRowsQuery<'_>) -> StorageResult<Vec<LeaderboardRow>> {
    let mut params = SqlParams::new();
    let limit = params.push(query.limit as i64);
    let offset = params.push(query.offset as i64);
    let order_sql = order_sql(query.order);
    let metric_sql = metric_sql(query.metric);
    let token_join = api_key_token_join(query.filters, &mut params);
    let sql = format!(
        "WITH aggregated AS ( \
            SELECT b.dimension_id, COALESCE(MAX(b.dimension_name), b.dimension_id) AS fallback_name, MAX(b.dimension_name) AS dimension_name, \
            COALESCE(SUM(b.request_count), 0)::bigint AS request_count, \
            COALESCE(SUM(b.input_tokens + b.output_tokens + b.cache_creation_tokens + b.cache_read_tokens), 0)::bigint AS total_tokens, \
            COALESCE(SUM(b.total_cost), 0) AS total_cost FROM dashboard_cost_analysis_buckets b {token_join} {} GROUP BY b.dimension_id \
        ), ranked AS ( \
            SELECT *, DENSE_RANK() OVER (ORDER BY {metric_sql} {order_sql}) AS rank FROM aggregated \
        ) \
        SELECT rank::bigint AS rank, dimension_id, fallback_name, dimension_name, request_count, total_tokens, total_cost \
        FROM ranked ORDER BY {metric_sql} {order_sql}, fallback_name ASC, dimension_id ASC LIMIT {limit} OFFSET {offset}",
        dimension_where(query.window, query.dimension_kind, query.filters, &mut params)
    );
    LeaderboardRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map_err(Into::into)
}

async fn dimension_total(
    store: &DashboardStore,
    window: &super::DashboardCostAnalysisWindow,
    dimension_kind: &str,
    filters: ApiKeyFilters,
) -> StorageResult<u64> {
    let mut params = SqlParams::new();
    let token_join = api_key_token_join(filters, &mut params);
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM (SELECT b.dimension_id FROM dashboard_cost_analysis_buckets b {token_join} {} GROUP BY b.dimension_id) items",
        dimension_where(window, dimension_kind, filters, &mut params)
    );
    CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .one(store.database().connection())
        .await?
        .map(|row| row.total.unwrap_or_default() as u64)
        .ok_or_else(|| StorageError::Database("dashboard cost leaderboard total returned no rows".into()))
}

fn api_key_token_join(filters: ApiKeyFilters, params: &mut SqlParams) -> String {
    if !filters.include_inactive && filters.exclude_admin {
        return format!(
            "JOIN api_tokens t ON t.id = b.dimension_id AND t.is_active = TRUE LEFT JOIN users u ON u.id = t.user_id AND u.role = {} ",
            params.push(ADMIN_ROLE.to_owned())
        );
    }
    if !filters.include_inactive {
        return "JOIN api_tokens t ON t.id = b.dimension_id AND t.is_active = TRUE ".into();
    }
    if filters.exclude_admin {
        return format!(
            "LEFT JOIN api_tokens t ON t.id = b.dimension_id LEFT JOIN users u ON u.id = t.user_id AND u.role = {} ",
            params.push(ADMIN_ROLE.to_owned())
        );
    }
    String::new()
}

async fn provider_rows(store: &DashboardStore, query: &DashboardProviderAggregationQuery) -> StorageResult<Vec<ProviderRow>> {
    let mut params = SqlParams::new();
    let limit = params.push(query.limit as i64);
    let sql = format!(
        "SELECT dimension_id, COALESCE(MAX(dimension_name), dimension_id) AS provider, \
        COALESCE(SUM(request_count), 0)::bigint AS request_count, COALESCE(SUM(success_count), 0)::bigint AS success_count, \
        COALESCE(SUM(failed_count), 0)::bigint AS failed_count, COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens, \
        COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, COALESCE(SUM(cache_creation_tokens), 0)::bigint AS cache_creation_tokens, \
        COALESCE(SUM(cache_read_tokens), 0)::bigint AS cache_read_tokens, COALESCE(SUM(total_tokens), 0)::bigint AS total_tokens, \
        COALESCE(SUM(total_cost), 0) AS total_cost, COALESCE(SUM(upstream_total_cost), 0) AS upstream_total_cost, \
        COALESCE(SUM(total_latency_ms), 0)::bigint AS total_latency_ms, COALESCE(SUM(latency_sample_count), 0)::bigint AS latency_sample_count, \
        {avg_response_headers} AS avg_response_headers_ms, {avg_first_byte} AS avg_first_byte_ms, \
        {avg_first_output} AS avg_first_output_ms \
        FROM dashboard_cost_analysis_buckets {where_sql} GROUP BY dimension_id ORDER BY request_count DESC, total_cost DESC, provider ASC LIMIT {limit}",
        avg_response_headers = avg_expr("response_headers_total_ms", "response_headers_sample_count"),
        avg_first_byte = avg_expr("first_byte_total_ms", "first_byte_sample_count"),
        avg_first_output = avg_expr("first_output_total_ms", "first_output_sample_count"),
        where_sql = base_dimension_where(&query.window, DIMENSION_PROVIDER, &mut params)
    );
    ProviderRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map_err(Into::into)
}

fn base_dimension_where(window: &super::DashboardCostAnalysisWindow, dimension_kind: &str, params: &mut SqlParams) -> String {
    format!(
        "WHERE bucket_started_at >= {} AND bucket_started_at < {} AND dimension_kind = {}",
        params.push(window.started_at),
        params.push(window.ended_at),
        params.push(dimension_kind.to_owned())
    )
}

fn dimension_where(window: &super::DashboardCostAnalysisWindow, dimension_kind: &str, filters: ApiKeyFilters, params: &mut SqlParams) -> String {
    let mut parts = vec![
        format!("b.bucket_started_at >= {}", params.push(window.started_at)),
        format!("b.bucket_started_at < {}", params.push(window.ended_at)),
        format!("b.dimension_kind = {}", params.push(dimension_kind.to_owned())),
    ];
    if filters.exclude_admin {
        parts.push("u.id IS NULL".into());
    }
    format!("WHERE {}", parts.join(" AND "))
}

fn provider_item(row: ProviderRow) -> DashboardProviderAggregationItem {
    let request_count = row.request_count.unwrap_or_default();
    let success_count = row.success_count.unwrap_or_default();
    let failed_count = row.failed_count.unwrap_or_default();
    let cache_read_tokens = row.cache_read_tokens.unwrap_or_default();
    let input_tokens = row.input_tokens.unwrap_or_default();
    let latency_samples = row.latency_sample_count.unwrap_or_default();
    DashboardProviderAggregationItem {
        provider_id: Some(row.dimension_id.clone()),
        provider_key: row.dimension_id,
        provider_identity_source: "provider_id".into(),
        provider: row.provider,
        request_count,
        total_tokens: row.total_tokens.unwrap_or_default(),
        effective_input_tokens: input_tokens,
        total_input_context: input_tokens + cache_read_tokens + row.cache_creation_tokens.unwrap_or_default(),
        output_tokens: row.output_tokens.unwrap_or_default(),
        total_cost: round_cost(row.total_cost.unwrap_or(Decimal::ZERO), COST_RESPONSE_SCALE),
        actual_cost: round_cost(row.upstream_total_cost.unwrap_or(Decimal::ZERO), COST_RESPONSE_SCALE),
        avg_response_time_ms: round_float(average(row.total_latency_ms.unwrap_or_default(), latency_samples), 2),
        avg_response_headers_ms: row.avg_response_headers_ms.map(|value| round_float(value, 2)),
        avg_first_byte_ms: row.avg_first_byte_ms.map(|value| round_float(value, 2)),
        avg_first_output_ms: row.avg_first_output_ms.map(|value| round_float(value, 2)),
        success_rate: round_float(percent(success_count, request_count), 2),
        error_count: failed_count,
        cache_creation_tokens: row.cache_creation_tokens.unwrap_or_default(),
        cache_read_tokens,
        cache_hit_rate: round_float(
            percent(
                cache_read_tokens,
                input_tokens + cache_read_tokens + row.cache_creation_tokens.unwrap_or_default(),
            ),
            2,
        ),
    }
}

fn metric_decimal(metric: DashboardUserStatsMetric, row: &LeaderboardRow) -> Decimal {
    match metric {
        DashboardUserStatsMetric::Requests => Decimal::from(row.request_count.unwrap_or_default()),
        DashboardUserStatsMetric::Tokens => Decimal::from(row.total_tokens.unwrap_or_default()),
        DashboardUserStatsMetric::Cost => row.total_cost.unwrap_or(Decimal::ZERO),
    }
}

fn metric_sql(metric: DashboardUserStatsMetric) -> &'static str {
    match metric {
        DashboardUserStatsMetric::Requests => "request_count",
        DashboardUserStatsMetric::Tokens => "total_tokens",
        DashboardUserStatsMetric::Cost => "total_cost",
    }
}

fn order_sql(order: DashboardSortOrder) -> &'static str {
    match order {
        DashboardSortOrder::Desc => "DESC",
        DashboardSortOrder::Asc => "ASC",
    }
}

fn forecast_points(
    last_history_date: Option<&str>,
    fallback_date: time::Date,
    forecast_days: u32,
    values: &[f64],
    slope: f64,
    intercept: f64,
) -> StorageResult<Vec<DashboardCostForecastPoint>> {
    let last_date = last_history_date.and_then(parse_date).unwrap_or(fallback_date);
    let mut output = Vec::new();
    for index in 0..forecast_days {
        let date = last_date
            .checked_add(time::Duration::days(i64::from(index + 1)))
            .ok_or_else(|| StorageError::Database("dashboard cost forecast date overflow".into()))?;
        let predicted = (slope * (values.len() + index as usize) as f64 + intercept).max(0.0);
        output.push(DashboardCostForecastPoint {
            date: date.to_string(),
            total_cost: round_cost(Decimal::from_f64_retain(predicted).unwrap_or(Decimal::ZERO), FORECAST_RESPONSE_SCALE),
        });
    }
    Ok(output)
}

fn linear_regression(values: &[f64]) -> (f64, f64) {
    let n = values.len();
    if n <= 1 {
        return (0.0, values.first().copied().unwrap_or(0.0));
    }
    let sum_x: f64 = (0..n).map(|value| value as f64).sum();
    let sum_y: f64 = values.iter().sum();
    let sum_x2: f64 = (0..n).map(|value| (value * value) as f64).sum();
    let sum_xy: f64 = values.iter().enumerate().map(|(index, value)| index as f64 * *value).sum();
    let n = n as f64;
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom == 0.0 {
        return (0.0, values.last().copied().unwrap_or(0.0));
    }
    ((n * sum_xy - sum_x * sum_y) / denom, (sum_y * sum_x2 - sum_x * sum_xy) / denom)
}

fn average(total: i64, count: i64) -> f64 {
    if count <= 0 {
        return 0.0;
    }
    total as f64 / count as f64
}

fn avg_expr(total_column: &str, count_column: &str) -> String {
    format!("COALESCE(SUM({total_column}), 0)::double precision / NULLIF(COALESCE(SUM({count_column}), 0), 0)::double precision")
}

fn percent(numerator: i64, denominator: i64) -> f64 {
    if denominator <= 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64 * PERCENT_MULTIPLIER
}

fn effective_estimated_full_cost(estimated_full_cost: Decimal, cache_read_cost: Decimal) -> Decimal {
    if estimated_full_cost <= Decimal::ZERO && cache_read_cost > Decimal::ZERO {
        return cache_read_cost * Decimal::from(CACHE_READ_ESTIMATE_MULTIPLIER);
    }
    estimated_full_cost
}

fn round_cost(value: Decimal, scale: u32) -> Decimal {
    value.round_dp(scale)
}

fn round_float(value: f64, scale: u32) -> f64 {
    let factor = 10_f64.powi(scale as i32);
    (value * factor).round() / factor
}

fn round_leaderboard_value(metric: DashboardUserStatsMetric, value: Decimal) -> Decimal {
    match metric {
        DashboardUserStatsMetric::Cost => round_cost(value, COST_RESPONSE_SCALE),
        DashboardUserStatsMetric::Requests | DashboardUserStatsMetric::Tokens => value,
    }
}

fn hour_bounds(value: time::OffsetDateTime) -> BucketBounds {
    let started_at = value
        .replace_minute(0)
        .and_then(|v| v.replace_second(0))
        .and_then(|v| v.replace_nanosecond(0))
        .unwrap_or(value);
    BucketBounds {
        started_at,
        ended_at: started_at + time::Duration::hours(1),
    }
}

fn shard(request_id: &str) -> i32 {
    let value = request_id.bytes().fold(0_u64, |acc, byte| acc.wrapping_mul(31).wrapping_add(u64::from(byte)));
    (value % SHARD_COUNT) as i32
}

fn is_terminal_status(status: &str) -> bool {
    status == TERMINAL_SUCCESS || status == TERMINAL_FAILED || status == TERMINAL_CANCELLED
}

#[derive(Clone, Debug)]
struct BucketContribution {
    request_id: String,
    token_id: Option<String>,
    token_name: Option<String>,
    provider_id: Option<String>,
    provider_name: Option<String>,
    success_count: i64,
    failed_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cache_creation_tokens: i64,
    cache_read_tokens: i64,
    total_tokens: i64,
    total_cost: Decimal,
    upstream_total_cost: Decimal,
    cache_read_cost: Decimal,
    cache_creation_cost: Decimal,
    estimated_full_cost: Decimal,
    total_latency_ms: i64,
    latency_sample_count: i64,
    response_headers_total_ms: i64,
    response_headers_sample_count: i64,
    first_byte_total_ms: i64,
    first_byte_sample_count: i64,
    first_sse_event_total_ms: i64,
    first_sse_event_sample_count: i64,
    first_output_total_ms: i64,
    first_output_sample_count: i64,
    sse_to_output_total_ms: i64,
    sse_to_output_sample_count: i64,
    created_at: time::OffsetDateTime,
}

impl BucketContribution {
    fn from_record(record: &request_records::Model) -> Option<Self> {
        if !is_terminal_status(&record.status) {
            return None;
        }
        let cache_read_tokens = token_context::cache_read_tokens(record);
        let input_price = record.input_price_per_million.unwrap_or(Decimal::ZERO);
        let estimated_full_cost = input_price * Decimal::from(cache_read_tokens) / Decimal::from(1_000_000_i64);
        let stage = StageLatencyContribution::new(record.response_headers_time_ms, record.first_sse_event_time_ms, record.first_output_time_ms);
        let first_byte_ms = non_negative(record.first_byte_time_ms);
        Some(Self {
            request_id: record.request_id.clone(),
            token_id: clean_optional(record.token_id.clone()),
            token_name: clean_optional(record.token_name_snapshot.clone()).or_else(|| clean_optional(record.token_prefix_snapshot.clone())),
            provider_id: clean_optional(record.provider_id.clone()),
            provider_name: clean_optional(record.provider_name_snapshot.clone()),
            success_count: i64::from(record.status == TERMINAL_SUCCESS),
            failed_count: i64::from(record.status == TERMINAL_FAILED || record.status == TERMINAL_CANCELLED),
            input_tokens: record.prompt_tokens.unwrap_or_default(),
            output_tokens: record.completion_tokens.unwrap_or_default(),
            cache_creation_tokens: token_context::cache_creation_tokens(record),
            cache_read_tokens,
            total_tokens: token_context::total_tokens(record),
            total_cost: record.total_cost.unwrap_or(Decimal::ZERO),
            upstream_total_cost: record.upstream_total_cost.unwrap_or(Decimal::ZERO),
            cache_read_cost: record.cache_read_cost.unwrap_or(Decimal::ZERO),
            cache_creation_cost: record.cache_creation_cost.unwrap_or(Decimal::ZERO),
            estimated_full_cost,
            total_latency_ms: record.total_latency_ms.unwrap_or_default(),
            latency_sample_count: i64::from(record.total_latency_ms.is_some()),
            response_headers_total_ms: StageLatencyContribution::total(stage.response_headers_ms),
            response_headers_sample_count: StageLatencyContribution::sample_count(stage.response_headers_ms),
            first_byte_total_ms: first_byte_ms.unwrap_or_default(),
            first_byte_sample_count: i64::from(first_byte_ms.is_some()),
            first_sse_event_total_ms: StageLatencyContribution::total(stage.first_sse_event_ms),
            first_sse_event_sample_count: StageLatencyContribution::sample_count(stage.first_sse_event_ms),
            first_output_total_ms: StageLatencyContribution::total(stage.first_output_ms),
            first_output_sample_count: StageLatencyContribution::sample_count(stage.first_output_ms),
            sse_to_output_total_ms: StageLatencyContribution::total(stage.sse_to_output_ms),
            sse_to_output_sample_count: StageLatencyContribution::sample_count(stage.sse_to_output_ms),
            created_at: record.created_at,
        })
    }

    fn dimensions(&self) -> Vec<Dimension> {
        let mut dimensions = vec![Dimension::new(DIMENSION_GLOBAL, GLOBAL_DIMENSION_ID, Some("Global".into()))];
        if let Some(provider_id) = &self.provider_id {
            dimensions.push(Dimension::new(DIMENSION_PROVIDER, provider_id, self.provider_name.clone()));
        }
        if let Some(token_id) = &self.token_id {
            dimensions.push(Dimension::new(DIMENSION_API_KEY, token_id, self.token_name.clone()));
        }
        dimensions
    }
}

#[derive(Clone, Debug)]
struct Dimension {
    kind: String,
    id: String,
    name: Option<String>,
}

impl Dimension {
    fn new(kind: &str, id: &str, name: Option<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            name,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BucketBounds {
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

fn non_negative(value: Option<i64>) -> Option<i64> {
    value.filter(|item| *item >= 0)
}

fn parse_date(value: &str) -> Option<time::Date> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u8>().ok()?;
    let day = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    time::Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()
}

#[derive(Debug, FromQueryResult)]
struct DailyCostRow {
    date: String,
    total_cost: Option<Decimal>,
}

#[derive(Debug, FromQueryResult)]
struct SavingsRow {
    cache_read_tokens: Option<i64>,
    cache_read_cost: Option<Decimal>,
    cache_creation_cost: Option<Decimal>,
    estimated_full_cost: Option<Decimal>,
}

#[derive(Debug, FromQueryResult)]
struct CountRow {
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct LeaderboardRow {
    rank: Option<i64>,
    dimension_id: String,
    fallback_name: String,
    dimension_name: Option<String>,
    request_count: Option<i64>,
    total_tokens: Option<i64>,
    total_cost: Option<Decimal>,
}

#[derive(Debug, FromQueryResult)]
struct ProviderRow {
    dimension_id: String,
    provider: String,
    request_count: Option<i64>,
    success_count: Option<i64>,
    failed_count: Option<i64>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    cache_creation_tokens: Option<i64>,
    cache_read_tokens: Option<i64>,
    total_tokens: Option<i64>,
    total_cost: Option<Decimal>,
    upstream_total_cost: Option<Decimal>,
    total_latency_ms: Option<i64>,
    latency_sample_count: Option<i64>,
    avg_response_headers_ms: Option<f64>,
    avg_first_byte_ms: Option<f64>,
    avg_first_output_ms: Option<f64>,
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::dashboard::DashboardUserStatsMetric;

    use super::{effective_estimated_full_cost, linear_regression, round_cost, round_leaderboard_value};

    #[test]
    fn savings_fallback_matches_aether_after_summary_aggregation() {
        let estimated = Decimal::ZERO;
        let cache_read = Decimal::new(2, 3);

        let result = effective_estimated_full_cost(estimated, cache_read);

        assert_eq!(result, Decimal::new(2, 2));
    }

    #[test]
    fn savings_keeps_positive_estimate_instead_of_per_row_fallback() {
        let estimated = Decimal::new(1, 3);
        let cache_read = Decimal::new(2, 3);

        let result = effective_estimated_full_cost(estimated, cache_read);

        assert_eq!(result, Decimal::new(1, 3));
    }

    #[test]
    fn forecast_linear_regression_matches_aether_formula() {
        let (slope, intercept) = linear_regression(&[1.0, 3.0, 5.0]);

        assert_eq!(slope, 2.0);
        assert_eq!(intercept, 1.0);
    }

    #[test]
    fn cost_values_round_to_aether_scale() {
        let value = Decimal::new(123456789, 8);

        assert_eq!(round_cost(value, 6), Decimal::new(1234568, 6));
    }

    #[test]
    fn leaderboard_rounds_only_cost_metric() {
        let cost = Decimal::new(123456789, 8);
        let tokens = Decimal::from(12345);

        assert_eq!(round_leaderboard_value(DashboardUserStatsMetric::Cost, cost), Decimal::new(1234568, 6));
        assert_eq!(round_leaderboard_value(DashboardUserStatsMetric::Tokens, tokens), tokens);
    }
}
