use rust_decimal::Decimal;

use super::overview::{BreakdownRow, SummaryRow, TimeseriesRow, breakdown_response, summary_response, timeseries_response};

#[test]
fn summary_response_calculates_cache_hit_rate_and_profit() {
    let response = summary_response(SummaryRow {
        request_count: Some(2),
        success_count: Some(1),
        failed_count: Some(1),
        active_count: Some(0),
        prompt_tokens: Some(75),
        cache_read_input_tokens: Some(25),
        total_tokens: Some(300),
        total_cost: Some(Decimal::new(12, 2)),
        upstream_total_cost: Some(Decimal::new(3, 2)),
        avg_latency_ms: Some(120.0),
        avg_ttfb_ms: Some(45.0),
        model_count: Some(2),
    }, true);

    assert_eq!(response.cache_hit_rate, 0.25);
    assert_eq!(response.profit, Decimal::new(9, 2));
    assert_eq!(response.profit_rate, 0.75);
    assert_eq!(response.avg_ttfb_ms, Some(45.0));
}

#[test]
fn timeseries_response_preserves_ttfb_cache_hit_rate_and_profit() {
    let response = timeseries_response(TimeseriesRow {
        bucket: "2026-04-28".into(),
        request_count: Some(4),
        success_count: Some(3),
        failed_count: Some(1),
        prompt_tokens: Some(90),
        cache_read_input_tokens: Some(10),
        total_tokens: Some(250),
        total_cost: Some(Decimal::new(55, 2)),
        upstream_total_cost: Some(Decimal::new(11, 2)),
        avg_latency_ms: Some(250.0),
        avg_ttfb_ms: Some(80.0),
    }, true);

    assert_eq!(response.cache_hit_rate, 0.1);
    assert_eq!(response.profit, Decimal::new(44, 2));
    assert_eq!(response.avg_ttfb_ms, Some(80.0));
}

#[test]
fn breakdown_response_preserves_average_latency_and_profit() {
    let response = breakdown_response(BreakdownRow {
        id: Some("provider-1".into()),
        name: "kedaya".into(),
        request_count: Some(255),
        total_tokens: Some(635_000_000),
        total_cost: Some(Decimal::new(18988, 2)),
        upstream_total_cost: Some(Decimal::new(9494, 2)),
        avg_latency_ms: Some(950.0),
    }, true);

    assert_eq!(response.profit, Decimal::new(9494, 2));
    assert_eq!(response.profit_rate, 0.5);
    assert_eq!(response.avg_latency_ms, Some(950.0));
}

#[test]
fn summary_response_hides_admin_costs_when_requested() {
    let response = summary_response(SummaryRow {
        request_count: Some(1),
        success_count: Some(1),
        failed_count: Some(0),
        active_count: Some(0),
        prompt_tokens: Some(10),
        cache_read_input_tokens: Some(0),
        total_tokens: Some(20),
        total_cost: Some(Decimal::new(12, 2)),
        upstream_total_cost: Some(Decimal::new(3, 2)),
        avg_latency_ms: None,
        avg_ttfb_ms: None,
        model_count: Some(1),
    }, false);

    assert_eq!(response.total_cost, Decimal::new(12, 2));
    assert_eq!(response.upstream_total_cost, Decimal::ZERO);
    assert_eq!(response.profit, Decimal::ZERO);
    assert_eq!(response.profit_rate, 0.0);
}
