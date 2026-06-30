use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{
    Database,
    dashboard::{DashboardBucketFilter, DashboardScopeFilter, DashboardStore, DashboardStoreOverviewQuery},
};
use types::{dashboard::DashboardPreset, pagination::PageRequest};

#[tokio::test]
async fn dashboard_overview_casts_latency_averages_to_double_precision() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[summary_row()]])
        .append_query_results([[summary_row()]])
        .append_query_results([[summary_row()]])
        .append_query_results([vec![timeseries_row()]])
        .append_query_results([daily_rows()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .into_connection();
    let store = DashboardStore::new(Database::new(connection.clone()));

    let response = store.overview(overview_query()).await.unwrap();

    assert_eq!(response.summary.avg_latency_ms, Some(125.5));
    assert_eq!(response.summary.avg_first_byte_ms, Some(42.25));
    assert_eq!(response.summary.avg_first_token_ms, Some(360.0));
    assert_eq!(response.summary.upstream_total_cost, Decimal::new(3, 2));
    assert_eq!(response.summary.profit, Decimal::new(9, 2));
    assert_eq!(response.timeseries[0].avg_latency_ms, Some(130.0));
    assert_eq!(response.timeseries[0].avg_first_token_ms, Some(365.0));
    assert_eq!(response.timeseries[0].upstream_total_cost, Decimal::new(3, 2));
    assert_eq!(response.daily.day_page.total, 1);
    assert_eq!(response.daily.day_page.items.len(), 1);
    assert_daily_costs(&response.daily);
    let logs = connection.into_transaction_log();
    let summary_sql = &logs[0].statements()[0].sql;
    let timeseries_sql = &logs[3].statements()[0].sql;
    let daily_sql = &logs[4].statements()[0].sql;
    let model_breakdown_sql = &logs[5].statements()[0].sql;
    assert_snapshot_metric_sql(summary_sql);
    assert_snapshot_metric_sql(timeseries_sql);
    assert_snapshot_metric_sql(daily_sql);
    assert!(summary_sql.contains("SUM(b.latency_total_ms)"), "{summary_sql}");
    assert!(summary_sql.contains("SUM(b.first_byte_total_ms)"), "{summary_sql}");
    assert!(summary_sql.contains("SUM(b.first_token_total_ms)"), "{summary_sql}");
    assert!(timeseries_sql.contains("SUM(b.latency_total_ms)"), "{timeseries_sql}");
    assert!(timeseries_sql.contains("SUM(b.first_token_total_ms)"), "{timeseries_sql}");
    assert!(timeseries_sql.contains("GROUP BY 1 ORDER BY 1 ASC"), "{timeseries_sql}");
    assert!(daily_sql.contains("SUM(b.upstream_total_cost)"), "{daily_sql}");
    assert!(daily_sql.contains("GROUP BY 1, 2, 3"), "{daily_sql}");
    assert!(model_breakdown_sql.contains("GROUP BY 1, 2"), "{model_breakdown_sql}");
    assert!(!model_breakdown_sql.contains("GROUP BY id, name"), "{model_breakdown_sql}");
}

#[tokio::test]
async fn dashboard_overview_uses_context_tokens_for_cache_usage() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[summary_row()]])
        .append_query_results([[summary_row()]])
        .append_query_results([[summary_row()]])
        .append_query_results([vec![timeseries_row()]])
        .append_query_results([daily_rows()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .into_connection();
    let store = DashboardStore::new(Database::new(connection.clone()));

    store.overview(overview_query()).await.unwrap();

    let logs = connection.into_transaction_log();
    let summary_sql = &logs[0].statements()[0].sql;
    let timeseries_sql = &logs[3].statements()[0].sql;
    let daily_sql = &logs[4].statements()[0].sql;
    assert_token_context_sql(summary_sql);
    assert_token_context_sql(timeseries_sql);
    assert_snapshot_metric_sql(daily_sql);
    assert!(daily_sql.contains("b.total_tokens"), "{daily_sql}");
}

fn assert_token_context_sql(sql: &str) {
    assert_snapshot_metric_sql(sql);
    assert!(sql.contains("b.cache_creation_input_tokens"), "{sql}");
    assert!(sql.contains("b.cache_read_input_tokens"), "{sql}");
    assert!(sql.contains("AS total_tokens"), "{sql}");
}

fn assert_snapshot_metric_sql(sql: &str) {
    assert!(sql.contains("FROM dashboard_request_metric_buckets b"), "{sql}");
    assert!(sql.contains("b.source_type = 'request'"), "{sql}");
    assert!(!sql.contains("FROM request_records"), "{sql}");
}

fn assert_daily_costs(daily: &types::dashboard::DashboardDailyStats) {
    let day = &daily.days[0];
    assert_eq!(day.total_cost, Decimal::new(30, 2));
    assert_eq!(day.upstream_total_cost, Decimal::new(12, 2));
    assert_eq!(day.profit, Decimal::new(18, 2));
    assert_eq!(day.profit_rate, 0.6);
    assert_eq!(day.model_breakdown[0].name, "model-a");
    assert_eq!(day.model_breakdown[0].upstream_total_cost, Decimal::new(10, 2));
    assert_eq!(day.provider_breakdown[0].name, "provider-a");
    assert_eq!(day.provider_breakdown[0].profit_rate, 0.6);
    assert_eq!(daily.model_summary[0].upstream_total_cost, Decimal::new(10, 2));
    assert_eq!(daily.provider_summary[0].upstream_total_cost, Decimal::new(10, 2));
}

fn overview_query() -> DashboardStoreOverviewQuery {
    DashboardStoreOverviewQuery {
        preset: DashboardPreset::Today,
        scope: DashboardScopeFilter::Me { user_id: "user-1".into() },
        started_at: ts(0),
        ended_at: ts(3_600),
        today_started_at: ts(0),
        today_ended_at: ts(3_600),
        monthly_started_at: ts(0),
        monthly_ended_at: ts(3_600),
        bucket: DashboardBucketFilter::Hour,
        include_admin_breakdowns: true,
        include_admin_costs: true,
        tz_offset_minutes: 0,
        daily_page: PageRequest { page: 1, page_size: 10 },
    }
}

fn summary_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("request_count", Value::from(2_i64)),
        ("success_count", Value::from(1_i64)),
        ("failed_count", Value::from(1_i64)),
        ("active_count", Value::from(0_i64)),
        ("prompt_tokens", Value::from(12_i64)),
        ("completion_tokens", Value::from(12_i64)),
        ("cache_creation_input_tokens", Value::from(0_i64)),
        ("cache_read_input_tokens", Value::from(0_i64)),
        ("total_tokens", Value::from(24_i64)),
        ("cache_creation_cost", Value::from(Decimal::ZERO)),
        ("cache_read_cost", Value::from(Decimal::ZERO)),
        ("total_cost", Value::from(Decimal::new(12, 2))),
        ("upstream_total_cost", Value::from(Decimal::new(3, 2))),
        ("avg_latency_ms", Value::from(125.5_f64)),
        ("avg_first_byte_ms", Value::from(42.25_f64)),
        ("avg_response_headers_ms", Value::from(38.0_f64)),
        ("avg_first_token_ms", Value::from(360.0_f64)),
        ("model_count", Value::from(1_i64)),
        ("provider_count", Value::from(1_i64)),
        ("user_count", Value::from(1_i64)),
        ("token_count", Value::from(1_i64)),
        ("failover_count", Value::from(0_i64)),
    ])
}

fn timeseries_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("bucket", Value::from("1970-01-01T00:00:00")),
        ("request_count", Value::from(2_i64)),
        ("success_count", Value::from(1_i64)),
        ("failed_count", Value::from(1_i64)),
        ("prompt_tokens", Value::from(12_i64)),
        ("cache_read_input_tokens", Value::from(0_i64)),
        ("total_tokens", Value::from(24_i64)),
        ("total_cost", Value::from(Decimal::new(12, 2))),
        ("upstream_total_cost", Value::from(Decimal::new(3, 2))),
        ("avg_latency_ms", Value::from(130.0_f64)),
        ("avg_first_byte_ms", Value::from(40.0_f64)),
        ("avg_response_headers_ms", Value::from(36.0_f64)),
        ("avg_first_token_ms", Value::from(365.0_f64)),
    ])
}

fn daily_rows() -> Vec<BTreeMap<&'static str, Value>> {
    vec![
        daily_row("model-a", "provider-a", Decimal::new(25, 2), Decimal::new(10, 2)),
        daily_row("model-b", "provider-b", Decimal::new(5, 2), Decimal::new(2, 2)),
    ]
}

fn daily_row(model_name: &'static str, provider_name: &'static str, total_cost: Decimal, upstream_total_cost: Decimal) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("date", Value::from(date())),
        ("model_name", Value::from(model_name)),
        ("provider_name", Value::from(provider_name)),
        ("request_count", Value::from(1_i64)),
        ("total_tokens", Value::from(12_i64)),
        ("total_cost", Value::from(total_cost)),
        ("upstream_total_cost", Value::from(upstream_total_cost)),
        ("latency_total_ms", Value::from(100.0_f64)),
        ("latency_sample_count", Value::from(1_i64)),
    ])
}

fn date() -> time::Date {
    time::Date::from_calendar_date(1970, time::Month::January, 1).unwrap()
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}
