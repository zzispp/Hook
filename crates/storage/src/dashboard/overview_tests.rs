use rust_decimal::Decimal;

use super::overview::{BreakdownRow, SummaryRow, TimeseriesRow, breakdown_response, summary_response, timeseries_response};

#[test]
fn summary_response_exposes_aether_dashboard_metrics() {
    let response = summary_response(
        SummaryRow {
            request_count: Some(10),
            success_count: Some(7),
            failed_count: Some(2),
            active_count: Some(1),
            prompt_tokens: Some(100),
            completion_tokens: Some(50),
            cache_creation_input_tokens: Some(30),
            cache_read_input_tokens: Some(20),
            total_tokens: Some(200),
            cache_creation_cost: Some(Decimal::new(3, 2)),
            cache_read_cost: Some(Decimal::new(1, 2)),
            total_cost: Some(Decimal::new(80, 2)),
            upstream_total_cost: Some(Decimal::new(30, 2)),
            avg_latency_ms: Some(200.0),
            avg_ttfb_ms: Some(75.0),
            avg_response_headers_ms: Some(40.0),
            avg_first_sse_event_ms: Some(60.0),
            avg_first_output_ms: Some(140.0),
            avg_sse_to_output_ms: Some(80.0),
            model_count: Some(4),
            provider_count: Some(3),
            user_count: Some(6),
            token_count: Some(5),
            failover_count: Some(2),
        },
        true,
    );

    assert_eq!(response.prompt_tokens, 100);
    assert_eq!(response.completion_tokens, 50);
    assert_eq!(response.cache_creation_input_tokens, 30);
    assert_eq!(response.cache_read_input_tokens, 20);
    assert_eq!(response.cache_creation_cost, Decimal::new(3, 2));
    assert_eq!(response.cache_read_cost, Decimal::new(1, 2));
    assert_eq!(response.error_rate, 2.0 / 9.0);
    assert_eq!(response.provider_count, 3);
    assert_eq!(response.user_count, 6);
    assert_eq!(response.token_count, 5);
    assert_eq!(response.failover_count, 2);
    assert_eq!(response.avg_first_output_ms, Some(140.0));
}

#[test]
fn summary_response_calculates_cache_hit_rate_and_profit() {
    let response = summary_response(
        SummaryRow {
            request_count: Some(2),
            success_count: Some(1),
            failed_count: Some(1),
            active_count: Some(0),
            prompt_tokens: Some(75),
            completion_tokens: Some(125),
            cache_creation_input_tokens: Some(0),
            cache_read_input_tokens: Some(25),
            total_tokens: Some(300),
            cache_creation_cost: Some(Decimal::ZERO),
            cache_read_cost: Some(Decimal::ZERO),
            total_cost: Some(Decimal::new(12, 2)),
            upstream_total_cost: Some(Decimal::new(3, 2)),
            avg_latency_ms: Some(120.0),
            avg_ttfb_ms: Some(45.0),
            avg_response_headers_ms: Some(20.0),
            avg_first_sse_event_ms: Some(35.0),
            avg_first_output_ms: Some(90.0),
            avg_sse_to_output_ms: Some(55.0),
            model_count: Some(2),
            provider_count: Some(1),
            user_count: Some(1),
            token_count: Some(1),
            failover_count: Some(0),
        },
        true,
    );

    assert_eq!(response.cache_hit_rate, 0.25);
    assert_eq!(response.profit, Decimal::new(9, 2));
    assert_eq!(response.profit_rate, 0.75);
    assert_eq!(response.avg_ttfb_ms, Some(45.0));
    assert_eq!(response.avg_sse_to_output_ms, Some(55.0));
}

#[test]
fn timeseries_response_preserves_ttfb_cache_hit_rate_and_profit() {
    let response = timeseries_response(
        TimeseriesRow {
            bucket: "2026-04-28".into(),
            request_count: Some(4),
            success_count: Some(3),
            failed_count: Some(1),
            prompt_tokens: Some(90),
            cache_creation_input_tokens: Some(0),
            cache_read_input_tokens: Some(10),
            total_tokens: Some(250),
            total_cost: Some(Decimal::new(55, 2)),
            upstream_total_cost: Some(Decimal::new(11, 2)),
            avg_latency_ms: Some(250.0),
            avg_ttfb_ms: Some(80.0),
            avg_response_headers_ms: Some(30.0),
            avg_first_sse_event_ms: Some(70.0),
            avg_first_output_ms: Some(180.0),
            avg_sse_to_output_ms: Some(110.0),
        },
        true,
    );

    assert_eq!(response.cache_hit_rate, 0.1);
    assert_eq!(response.profit, Decimal::new(44, 2));
    assert_eq!(response.avg_ttfb_ms, Some(80.0));
    assert_eq!(response.avg_first_output_ms, Some(180.0));
}

#[test]
fn timeseries_response_includes_cache_creation_in_hit_rate_denominator() {
    let response = timeseries_response(
        TimeseriesRow {
            bucket: "2026-04-28".into(),
            request_count: Some(4),
            success_count: Some(3),
            failed_count: Some(1),
            prompt_tokens: Some(70),
            cache_creation_input_tokens: Some(20),
            cache_read_input_tokens: Some(10),
            total_tokens: Some(250),
            total_cost: Some(Decimal::new(55, 2)),
            upstream_total_cost: Some(Decimal::new(11, 2)),
            avg_latency_ms: Some(250.0),
            avg_ttfb_ms: Some(80.0),
            avg_response_headers_ms: None,
            avg_first_sse_event_ms: None,
            avg_first_output_ms: None,
            avg_sse_to_output_ms: None,
        },
        true,
    );

    assert_eq!(response.cache_hit_rate, 0.1);
}

#[test]
fn breakdown_response_preserves_average_latency_and_profit() {
    let response = breakdown_response(
        BreakdownRow {
            id: Some("provider-1".into()),
            name: "kedaya".into(),
            request_count: Some(255),
            total_tokens: Some(635_000_000),
            total_cost: Some(Decimal::new(18988, 2)),
            upstream_total_cost: Some(Decimal::new(9494, 2)),
            avg_latency_ms: Some(950.0),
            avg_response_headers_ms: Some(100.0),
            avg_first_sse_event_ms: Some(175.0),
            avg_first_output_ms: Some(425.0),
            avg_sse_to_output_ms: Some(250.0),
        },
        true,
    );

    assert_eq!(response.profit, Decimal::new(9494, 2));
    assert_eq!(response.profit_rate, 0.5);
    assert_eq!(response.avg_latency_ms, Some(950.0));
    assert_eq!(response.avg_first_output_ms, Some(425.0));
}

#[test]
fn summary_response_hides_admin_costs_when_requested() {
    let response = summary_response(
        SummaryRow {
            request_count: Some(1),
            success_count: Some(1),
            failed_count: Some(0),
            active_count: Some(0),
            prompt_tokens: Some(10),
            completion_tokens: Some(10),
            cache_creation_input_tokens: Some(0),
            cache_read_input_tokens: Some(0),
            total_tokens: Some(20),
            cache_creation_cost: Some(Decimal::ZERO),
            cache_read_cost: Some(Decimal::ZERO),
            total_cost: Some(Decimal::new(12, 2)),
            upstream_total_cost: Some(Decimal::new(3, 2)),
            avg_latency_ms: None,
            avg_ttfb_ms: None,
            avg_response_headers_ms: None,
            avg_first_sse_event_ms: None,
            avg_first_output_ms: None,
            avg_sse_to_output_ms: None,
            model_count: Some(1),
            provider_count: Some(1),
            user_count: Some(1),
            token_count: Some(1),
            failover_count: Some(0),
        },
        false,
    );

    assert_eq!(response.total_cost, Decimal::new(12, 2));
    assert_eq!(response.upstream_total_cost, Decimal::ZERO);
    assert_eq!(response.profit, Decimal::ZERO);
    assert_eq!(response.profit_rate, 0.0);
}
