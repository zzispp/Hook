use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, TransactionTrait};
use types::dashboard::{
    DashboardUserStatsLeaderboardItem, DashboardUserStatsLeaderboardResponse, DashboardUserStatsMetric, DashboardUserStatsTimeSeriesPoint,
    DashboardUserUsageStatsResponse,
};

use crate::{StorageError, StorageResult, provider::record::request_records};

use super::{
    DashboardStore, DashboardUserStatsBucket, DashboardUserStatsLeaderboardQuery, DashboardUserStatsTimeSeriesQuery, DashboardUserUsageStatsQuery,
    scope::SqlParams,
};

const GRANULARITY_HOUR: &str = "hour";
const GRANULARITY_DAY: &str = "day";
const STATUS_SUCCESS: &str = "success";
const STATUS_FAILED: &str = "failed";
const STATUS_CANCELLED: &str = "cancelled";

pub async fn sync_user_usage_buckets(
    connection: &sea_orm::DatabaseConnection,
    old_record: &request_records::Model,
    new_record: &request_records::Model,
) -> StorageResult<()> {
    let old_contribution = contribution(old_record);
    let new_contribution = contribution(new_record);
    if old_contribution.is_none() && new_contribution.is_none() {
        return Ok(());
    }
    let tx = connection.begin().await?;
    apply_record_delta(&tx, old_contribution.as_ref(), -1).await?;
    apply_record_delta(&tx, new_contribution.as_ref(), 1).await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn leaderboard(
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

pub(super) async fn summary(store: &DashboardStore, query: DashboardUserUsageStatsQuery) -> StorageResult<DashboardUserUsageStatsResponse> {
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

pub(super) async fn time_series(
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
        SELECT rank::bigint AS rank, id, name, requests, tokens, cost, {order_column} AS value \
        FROM ranked \
        ORDER BY {order_column} DESC, name ASC, id ASC \
        LIMIT {limit} OFFSET {offset}"
    );
    LeaderboardRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map_err(Into::into)
}

async fn apply_record_delta<C>(connection: &C, contribution: Option<&BucketContribution>, multiplier: i64) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let Some(contribution) = contribution else {
        return Ok(());
    };
    upsert_bucket_delta(connection, contribution, GRANULARITY_HOUR, hour_bounds(contribution.created_at), multiplier).await?;
    upsert_bucket_delta(connection, contribution, GRANULARITY_DAY, day_bounds(contribution.created_at), multiplier).await
}

async fn upsert_bucket_delta<C>(
    connection: &C,
    contribution: &BucketContribution,
    granularity: &str,
    bounds: BucketBounds,
    multiplier: i64,
) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let mut params = SqlParams::new();
    let sql = format!(
        "INSERT INTO dashboard_user_usage_buckets \
        (id, bucket_granularity, bucket_started_at, bucket_ended_at, user_id, username, request_count, success_count, failed_count, total_tokens, total_cost, total_latency_ms, created_at, updated_at) \
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) \
        ON CONFLICT (bucket_granularity, bucket_started_at, user_id) DO UPDATE SET \
        username = EXCLUDED.username, \
        request_count = dashboard_user_usage_buckets.request_count + EXCLUDED.request_count, \
        success_count = dashboard_user_usage_buckets.success_count + EXCLUDED.success_count, \
        failed_count = dashboard_user_usage_buckets.failed_count + EXCLUDED.failed_count, \
        total_tokens = dashboard_user_usage_buckets.total_tokens + EXCLUDED.total_tokens, \
        total_cost = dashboard_user_usage_buckets.total_cost + EXCLUDED.total_cost, \
        total_latency_ms = dashboard_user_usage_buckets.total_latency_ms + EXCLUDED.total_latency_ms, \
        updated_at = EXCLUDED.updated_at",
        params.push(uuid::Uuid::now_v7().to_string()),
        params.push(granularity.to_owned()),
        params.push(bounds.started_at),
        params.push(bounds.ended_at),
        params.push(contribution.user_id.clone()),
        params.push(contribution.username.clone()),
        params.push(multiplier),
        params.push(contribution.success_count * multiplier),
        params.push(contribution.failed_count * multiplier),
        params.push(contribution.total_tokens * multiplier),
        params.push(contribution.total_cost * Decimal::from(multiplier)),
        params.push(contribution.total_latency_ms * multiplier),
        params.push(time::OffsetDateTime::now_utc()),
        params.push(time::OffsetDateTime::now_utc())
    );
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

fn contribution(record: &request_records::Model) -> Option<BucketContribution> {
    if !is_terminal_status(&record.status) {
        return None;
    }
    let user_id = record.user_id_snapshot.as_ref()?.trim();
    if user_id.is_empty() {
        return None;
    }
    Some(BucketContribution {
        user_id: user_id.to_owned(),
        username: record.username_snapshot.clone(),
        success_count: i64::from(record.status == STATUS_SUCCESS),
        failed_count: i64::from(record.status == STATUS_FAILED || record.status == STATUS_CANCELLED),
        total_tokens: total_tokens(record),
        total_cost: record.total_cost.unwrap_or(Decimal::ZERO),
        total_latency_ms: record.total_latency_ms.unwrap_or_default(),
        created_at: record.created_at,
    })
}

fn bucket_where(window: &super::DashboardUserStatsStoreWindow, granularity: &str, user_id: Option<&str>, params: &mut SqlParams) -> String {
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

fn total_tokens(record: &request_records::Model) -> i64 {
    record
        .total_tokens
        .unwrap_or_else(|| record.prompt_tokens.unwrap_or_default() + record.completion_tokens.unwrap_or_default())
}

fn hour_bounds(value: time::OffsetDateTime) -> BucketBounds {
    let started_at = value.replace_minute(0).and_then(|v| v.replace_second(0)).and_then(|v| v.replace_nanosecond(0)).unwrap_or(value);
    BucketBounds {
        started_at,
        ended_at: started_at + time::Duration::hours(1),
    }
}

fn day_bounds(value: time::OffsetDateTime) -> BucketBounds {
    let date = value.date();
    let started_at = date.midnight().assume_utc();
    BucketBounds {
        started_at,
        ended_at: started_at + time::Duration::days(1),
    }
}

fn metric_column(metric: DashboardUserStatsMetric) -> &'static str {
    match metric {
        DashboardUserStatsMetric::Requests => "requests",
        DashboardUserStatsMetric::Tokens => "tokens",
        DashboardUserStatsMetric::Cost => "cost",
    }
}

fn bucket_granularity(bucket: DashboardUserStatsBucket) -> &'static str {
    match bucket {
        DashboardUserStatsBucket::Hour => GRANULARITY_HOUR,
        DashboardUserStatsBucket::Day => GRANULARITY_DAY,
    }
}

fn is_terminal_status(status: &str) -> bool {
    status == STATUS_SUCCESS || status == STATUS_FAILED || status == STATUS_CANCELLED
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

#[derive(Clone, Debug)]
struct BucketContribution {
    user_id: String,
    username: Option<String>,
    success_count: i64,
    failed_count: i64,
    total_tokens: i64,
    total_cost: Decimal,
    total_latency_ms: i64,
    created_at: time::OffsetDateTime,
}

#[derive(Clone, Copy, Debug)]
struct BucketBounds {
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
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
