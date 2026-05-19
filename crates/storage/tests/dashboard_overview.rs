use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{
    Database,
    dashboard::{DashboardBucketFilter, DashboardScopeFilter, DashboardStore, DashboardStoreOverviewQuery},
};
use types::dashboard::DashboardPreset;

#[tokio::test]
async fn dashboard_overview_casts_latency_averages_to_double_precision() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[summary_row()]])
        .append_query_results([vec![timeseries_row()]])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .into_connection();
    let store = DashboardStore::new(Database::new(connection.clone()));

    let response = store.overview(overview_query()).await.unwrap();

    assert_eq!(response.summary.avg_latency_ms, Some(125.5));
    assert_eq!(response.summary.avg_ttfb_ms, Some(42.25));
    assert_eq!(response.timeseries[0].avg_latency_ms, Some(130.0));
    let logs = connection.into_transaction_log();
    let summary_sql = &logs[0].statements()[0].sql;
    let timeseries_sql = &logs[1].statements()[0].sql;
    assert!(summary_sql.contains("AVG(r.total_latency_ms::double precision)"), "{summary_sql}");
    assert!(summary_sql.contains("AVG(r.first_byte_time_ms::double precision)"), "{summary_sql}");
    assert!(timeseries_sql.contains("AVG(r.total_latency_ms::double precision)"), "{timeseries_sql}");
}

fn overview_query() -> DashboardStoreOverviewQuery {
    DashboardStoreOverviewQuery {
        preset: DashboardPreset::Today,
        scope: DashboardScopeFilter::Me { user_id: "user-1".into() },
        started_at: ts(0),
        ended_at: ts(3_600),
        bucket: DashboardBucketFilter::Hour,
        include_admin_breakdowns: false,
        tz_offset_minutes: 0,
    }
}

fn summary_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("request_count", Value::from(2_i64)),
        ("success_count", Value::from(1_i64)),
        ("failed_count", Value::from(1_i64)),
        ("active_count", Value::from(0_i64)),
        ("total_tokens", Value::from(24_i64)),
        ("total_cost", Value::from(Decimal::new(12, 2))),
        ("avg_latency_ms", Value::from(125.5_f64)),
        ("avg_ttfb_ms", Value::from(42.25_f64)),
        ("model_count", Value::from(1_i64)),
    ])
}

fn timeseries_row() -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("bucket", Value::from("1970-01-01T00:00:00")),
        ("request_count", Value::from(2_i64)),
        ("success_count", Value::from(1_i64)),
        ("failed_count", Value::from(1_i64)),
        ("total_tokens", Value::from(24_i64)),
        ("total_cost", Value::from(Decimal::new(12, 2))),
        ("avg_latency_ms", Value::from(130.0_f64)),
    ])
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}
