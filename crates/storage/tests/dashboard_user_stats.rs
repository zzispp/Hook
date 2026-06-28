use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{
    Database,
    dashboard::{
        DashboardStore, DashboardUserStatsBucket, DashboardUserStatsLeaderboardQuery, DashboardUserStatsStoreWindow, DashboardUserStatsTimeSeriesQuery,
        DashboardUserUsageStatsQuery,
    },
};
use types::dashboard::DashboardUserStatsMetric;

#[tokio::test]
async fn leaderboard_requests_metric_returns_request_value_and_ordering() {
    assert_leaderboard_metric(DashboardUserStatsMetric::Requests, "requests", "requests::numeric", Decimal::from(42)).await;
}

#[tokio::test]
async fn leaderboard_tokens_metric_returns_token_value_and_ordering() {
    assert_leaderboard_metric(DashboardUserStatsMetric::Tokens, "tokens", "tokens::numeric", Decimal::from(4_096)).await;
}

#[tokio::test]
async fn leaderboard_cost_metric_returns_cost_value_and_ordering() {
    assert_leaderboard_metric(DashboardUserStatsMetric::Cost, "cost", "cost", Decimal::new(1234, 2)).await;
}

async fn assert_leaderboard_metric(metric: DashboardUserStatsMetric, order_column: &str, value_column: &str, expected_value: Decimal) {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row()]])
        .append_query_results([[leaderboard_row(expected_value)]])
        .into_connection();
    let store = DashboardStore::new(Database::new(connection.clone()));

    let response = store.user_stats_leaderboard(query(metric)).await.unwrap();

    assert_eq!(response.metric, metric);
    assert_eq!(response.total, 1);
    assert_eq!(response.items[0].value, expected_value);
    assert_eq!(response.items[0].requests, 42);
    assert_eq!(response.items[0].tokens, 4_096);
    assert_eq!(response.items[0].cost, Decimal::new(1234, 2));
    let logs = connection.into_transaction_log();
    let sql = &logs[1].statements()[0].sql;
    assert!(sql.contains("requests, tokens, cost"), "{sql}");
    assert!(sql.contains(&format!("{value_column} AS value")), "{sql}");
    assert!(sql.contains(&format!("ORDER BY {order_column} DESC")), "{sql}");
}

#[tokio::test]
async fn leaderboard_request_and_token_values_are_selected_as_decimal() {
    assert_leaderboard_value_cast(DashboardUserStatsMetric::Requests, "requests").await;
    assert_leaderboard_value_cast(DashboardUserStatsMetric::Tokens, "tokens").await;
}

async fn assert_leaderboard_value_cast(metric: DashboardUserStatsMetric, order_column: &str) {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row()]])
        .append_query_results([[leaderboard_row(Decimal::ONE)]])
        .into_connection();
    let store = DashboardStore::new(Database::new(connection.clone()));

    store.user_stats_leaderboard(query(metric)).await.unwrap();

    let logs = connection.into_transaction_log();
    let sql = &logs[1].statements()[0].sql;
    assert!(sql.contains(&format!("{order_column}::numeric AS value")), "{sql}");
}

#[tokio::test]
async fn user_usage_summary_and_time_series_return_first_output_latency() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[usage_summary_row()]])
        .append_query_results([vec![time_series_row()]])
        .into_connection();
    let store = DashboardStore::new(Database::new(connection.clone()));

    let summary = store.user_usage_stats(usage_query()).await.unwrap();
    let series = store.user_stats_time_series(series_query()).await.unwrap();

    assert_eq!(summary.avg_first_output_ms, Some(350.0));
    assert_eq!(series[0].avg_first_output_ms, Some(370.0));
    let logs = connection.into_transaction_log();
    let summary_sql = &logs[0].statements()[0].sql;
    let series_sql = &logs[1].statements()[0].sql;
    assert!(summary_sql.contains("first_output_total_ms"), "{summary_sql}");
    assert!(summary_sql.contains("first_output_sample_count"), "{summary_sql}");
    assert!(series_sql.contains("first_output_total_ms"), "{series_sql}");
}

fn query(metric: DashboardUserStatsMetric) -> DashboardUserStatsLeaderboardQuery {
    DashboardUserStatsLeaderboardQuery {
        window: DashboardUserStatsStoreWindow {
            start_date: date(1),
            end_date: date(2),
            started_at: timestamp(1),
            ended_at: timestamp(2),
        },
        metric,
        limit: 10,
        offset: 0,
    }
}

fn usage_query() -> DashboardUserUsageStatsQuery {
    DashboardUserUsageStatsQuery {
        window: window(),
        user_id: Some("user-1".into()),
    }
}

fn series_query() -> DashboardUserStatsTimeSeriesQuery {
    DashboardUserStatsTimeSeriesQuery {
        window: window(),
        bucket: DashboardUserStatsBucket::Day,
        user_id: Some("user-1".into()),
    }
}

fn window() -> DashboardUserStatsStoreWindow {
    DashboardUserStatsStoreWindow {
        start_date: date(1),
        end_date: date(2),
        started_at: timestamp(1),
        ended_at: timestamp(2),
    }
}

fn count_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("total", Value::from(1_i64))])
}

fn leaderboard_row(value: Decimal) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("rank", Value::from(1_i64)),
        ("id", Value::from("user-1")),
        ("name", Value::from("Alice")),
        ("requests", Value::from(42_i64)),
        ("tokens", Value::from(4_096_i64)),
        ("cost", Value::from(Decimal::new(1234, 2))),
        ("value", Value::from(value)),
    ])
}

fn usage_summary_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("total_requests", Value::from(2_i64)),
        ("total_tokens", Value::from(4_096_i64)),
        ("total_cost", Value::from(Decimal::new(1234, 2))),
        ("failed_count", Value::from(1_i64)),
        ("avg_first_output_ms", Value::from(350.0_f64)),
    ])
}

fn time_series_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("date", Value::from("2026-05-01")),
        ("total_cost", Value::from(Decimal::new(1234, 2))),
        ("total_requests", Value::from(2_i64)),
        ("total_tokens", Value::from(4_096_i64)),
        ("avg_first_output_ms", Value::from(370.0_f64)),
    ])
}

fn date(day: u8) -> time::Date {
    time::Date::from_calendar_date(2026, time::Month::May, day).unwrap()
}

fn timestamp(day: u8) -> time::OffsetDateTime {
    date(day).with_hms(0, 0, 0).unwrap().assume_utc()
}
