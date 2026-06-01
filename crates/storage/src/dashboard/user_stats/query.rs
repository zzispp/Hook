use rust_decimal::Decimal;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use types::dashboard::{
    DashboardUserStatsLeaderboardItem, DashboardUserStatsLeaderboardResponse, DashboardUserStatsMetric, DashboardUserStatsTimeSeriesPoint,
    DashboardUserUsageStatsResponse,
};

use crate::{StorageError, StorageResult};

use super::GRANULARITY_DAY;
use crate::dashboard::{
    DashboardStore, DashboardUserStatsBucket, DashboardUserStatsLeaderboardQuery, DashboardUserStatsTimeSeriesQuery, DashboardUserUsageStatsQuery,
    scope::SqlParams,
};

pub(in crate::dashboard) async fn leaderboard(
    store: &DashboardStore,
    query: DashboardUserStatsLeaderboardQuery,
) -> StorageResult<DashboardUserStatsLeaderboardResponse> {
    let total = leaderboard_total(store, &query).await?;
    let rows = leaderboard_rows(store, &query).await?;
    let items = rows.into_iter().map(leaderboard_item).collect();
    Ok(DashboardUserStatsLeaderboardResponse {
        items,
        total,
        metric: query.metric,
        start_date: query.window.start_date.to_string(),
        end_date: query.window.end_date.to_string(),
    })
}

pub(in crate::dashboard) async fn summary(store: &DashboardStore, query: DashboardUserUsageStatsQuery) -> StorageResult<DashboardUserUsageStatsResponse> {
    let mut params = SqlParams::new();
    let where_sql = bucket_where(&query.window, GRANULARITY_DAY, query.user_id.as_deref(), &mut params);
    let sql = format!(
        "SELECT \
        COALESCE(SUM(request_count), 0)::bigint AS total_requests, \
        COALESCE(SUM(total_tokens), 0)::bigint AS total_tokens, \
        COALESCE(SUM(total_cost), 0) AS total_cost, \
        COALESCE(SUM(failed_count), 0)::bigint AS failed_count \
        FROM dashboard_user_usage_buckets {where_sql}"
    );
    let row = SummaryRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .one(store.database().connection())
        .await?
        .ok_or_else(|| StorageError::Database("dashboard user usage summary returned no rows".into()))?;
    Ok(summary_response(row))
}

pub(in crate::dashboard) async fn time_series(
    store: &DashboardStore,
    query: DashboardUserStatsTimeSeriesQuery,
) -> StorageResult<Vec<DashboardUserStatsTimeSeriesPoint>> {
    let mut params = SqlParams::new();
    let where_sql = bucket_where(&query.window, bucket_granularity(query.bucket), query.user_id.as_deref(), &mut params);
    let sql = format!(
        "SELECT to_char(bucket_started_at AT TIME ZONE 'UTC', {}) AS date, \
        COALESCE(SUM(total_cost), 0) AS total_cost, \
        COALESCE(SUM(request_count), 0)::bigint AS total_requests, \
        COALESCE(SUM(total_tokens), 0)::bigint AS total_tokens \
        FROM dashboard_user_usage_buckets {where_sql} \
        GROUP BY date \
        ORDER BY date ASC",
        params.push(date_format(query.bucket))
    );
    TimeSeriesRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map(|rows| rows.into_iter().map(time_series_point).collect())
        .map_err(Into::into)
}

async fn leaderboard_total(store: &DashboardStore, query: &DashboardUserStatsLeaderboardQuery) -> StorageResult<u64> {
    let mut params = SqlParams::new();
    let where_sql = bucket_where(&query.window, GRANULARITY_DAY, None, &mut params);
    let sql = format!("SELECT COUNT(*)::bigint AS total FROM (SELECT user_id FROM dashboard_user_usage_buckets {where_sql} GROUP BY user_id) users");
    CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .one(store.database().connection())
        .await?
        .map(|row| row.total.unwrap_or_default() as u64)
        .ok_or_else(|| StorageError::Database("dashboard user leaderboard total returned no rows".into()))
}

async fn leaderboard_rows(store: &DashboardStore, query: &DashboardUserStatsLeaderboardQuery) -> StorageResult<Vec<LeaderboardRow>> {
    let mut params = SqlParams::new();
    let where_sql = bucket_where(&query.window, GRANULARITY_DAY, None, &mut params);
    let order_column = metric_column(query.metric);
    let value_column = metric_value_column(query.metric);
    let limit = params.push(query.limit as i64);
    let offset = params.push(query.offset as i64);
    let sql = format!(
        "WITH aggregated AS ( \
            SELECT user_id AS id, \
            COALESCE(MAX(username), user_id) AS name, \
            COALESCE(SUM(request_count), 0)::bigint AS requests, \
            COALESCE(SUM(total_tokens), 0)::bigint AS tokens, \
            COALESCE(SUM(total_cost), 0) AS cost \
            FROM dashboard_user_usage_buckets {where_sql} \
            GROUP BY user_id \
        ), ranked AS ( \
            SELECT *, DENSE_RANK() OVER (ORDER BY {order_column} DESC) AS rank \
            FROM aggregated \
        ) \
        SELECT rank::bigint AS rank, id, name, requests, tokens, cost, {value_column} AS value \
        FROM ranked \
        ORDER BY {order_column} DESC, name ASC, id ASC \
        LIMIT {limit} OFFSET {offset}"
    );
    LeaderboardRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map_err(Into::into)
}

fn bucket_where(window: &crate::dashboard::DashboardUserStatsStoreWindow, granularity: &str, user_id: Option<&str>, params: &mut SqlParams) -> String {
    let mut filters = vec![
        format!("bucket_started_at >= {}", params.push(window.started_at)),
        format!("bucket_started_at < {}", params.push(window.ended_at)),
        format!("bucket_granularity = {}", params.push(granularity.to_owned())),
    ];
    if let Some(user_id) = user_id {
        filters.push(format!("user_id = {}", params.push(user_id.to_owned())));
    }
    format!("WHERE {}", filters.join(" AND "))
}

fn leaderboard_item(row: LeaderboardRow) -> DashboardUserStatsLeaderboardItem {
    DashboardUserStatsLeaderboardItem {
        rank: row.rank.unwrap_or_default() as u64,
        id: row.id,
        name: row.name,
        value: row.value.unwrap_or(Decimal::ZERO),
        requests: row.requests.unwrap_or_default(),
        tokens: row.tokens.unwrap_or_default(),
        cost: row.cost.unwrap_or(Decimal::ZERO),
    }
}

fn summary_response(row: SummaryRow) -> DashboardUserUsageStatsResponse {
    let total_requests = row.total_requests.unwrap_or_default();
    let failed_count = row.failed_count.unwrap_or_default();
    DashboardUserUsageStatsResponse {
        total_requests,
        total_tokens: row.total_tokens.unwrap_or_default(),
        total_cost: row.total_cost.unwrap_or(Decimal::ZERO),
        error_rate: error_rate(failed_count, total_requests),
    }
}

fn time_series_point(row: TimeSeriesRow) -> DashboardUserStatsTimeSeriesPoint {
    DashboardUserStatsTimeSeriesPoint {
        date: row.date,
        total_cost: row.total_cost.unwrap_or(Decimal::ZERO),
        total_requests: row.total_requests.unwrap_or_default(),
        total_tokens: row.total_tokens.unwrap_or_default(),
    }
}

fn metric_column(metric: DashboardUserStatsMetric) -> &'static str {
    match metric {
        DashboardUserStatsMetric::Requests => "requests",
        DashboardUserStatsMetric::Tokens => "tokens",
        DashboardUserStatsMetric::Cost => "cost",
    }
}

fn metric_value_column(metric: DashboardUserStatsMetric) -> &'static str {
    match metric {
        DashboardUserStatsMetric::Requests => "requests::numeric",
        DashboardUserStatsMetric::Tokens => "tokens::numeric",
        DashboardUserStatsMetric::Cost => "cost",
    }
}

fn bucket_granularity(bucket: DashboardUserStatsBucket) -> &'static str {
    match bucket {
        DashboardUserStatsBucket::Hour => super::GRANULARITY_HOUR,
        DashboardUserStatsBucket::Day => GRANULARITY_DAY,
    }
}

fn date_format(bucket: DashboardUserStatsBucket) -> &'static str {
    match bucket {
        DashboardUserStatsBucket::Hour => "YYYY-MM-DD\"T\"HH24:MI:SS",
        DashboardUserStatsBucket::Day => "YYYY-MM-DD",
    }
}

fn error_rate(failed_count: i64, request_count: i64) -> f64 {
    if request_count == 0 {
        return 0.0;
    }
    failed_count as f64 / request_count as f64 * 100.0
}

#[derive(Clone, Debug, FromQueryResult)]
struct CountRow {
    total: Option<i64>,
}

#[derive(Clone, Debug, FromQueryResult)]
struct LeaderboardRow {
    rank: Option<i64>,
    id: String,
    name: String,
    value: Option<Decimal>,
    requests: Option<i64>,
    tokens: Option<i64>,
    cost: Option<Decimal>,
}

#[derive(Clone, Debug, FromQueryResult)]
struct SummaryRow {
    total_requests: Option<i64>,
    total_tokens: Option<i64>,
    total_cost: Option<Decimal>,
    failed_count: Option<i64>,
}

#[derive(Clone, Debug, FromQueryResult)]
struct TimeSeriesRow {
    date: String,
    total_cost: Option<Decimal>,
    total_requests: Option<i64>,
    total_tokens: Option<i64>,
}
