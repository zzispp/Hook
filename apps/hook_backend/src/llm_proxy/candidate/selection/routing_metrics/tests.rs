use std::collections::HashMap;

use storage::provider::RoutingMetricRecord;
use types::provider::{RouteIdentity, RoutingMetricSnapshot, RoutingMetricSource, RoutingMetricWindow, RoutingPriorSource};

use super::{MetricCatalog, RouteFingerprints, RoutingMetricsSnapshot};

const CURRENT_ROUTE_FINGERPRINT: &str = "route-fingerprint";
const CURRENT_PRICE_FINGERPRINT: &str = "price-fingerprint";

#[test]
fn resolve_prefers_exact_route_metric() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let catalog = catalog(vec![record(route.clone(), 25)]);

    let resolved = catalog.resolve(&route, fingerprints(), 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Exact);
    assert_eq!(resolved.prior_source, RoutingPriorSource::ExactRoute);
    assert_eq!(resolved.prior_sample_count, 25);
}

#[test]
fn resolve_uses_prior_when_exact_route_fingerprint_mismatches() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let catalog = catalog(vec![record_with_fingerprints(route.clone(), 25, "old-route", "old-price")]);

    let resolved = catalog.resolve(&route, fingerprints(), 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Prior);
    assert_eq!(resolved.prior_source, RoutingPriorSource::ProviderModelFormat);
    assert_eq!(resolved.prior_sample_count, 25);
}

#[test]
fn resolve_keeps_multiple_fingerprint_versions_for_same_route() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let catalog = catalog(vec![
        record_with_fingerprints(route.clone(), 10, "old-route", "old-price"),
        record(route.clone(), 25),
    ]);

    let resolved = catalog.resolve(&route, fingerprints(), 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Exact);
    assert_eq!(resolved.prior_sample_count, 25);
}

#[test]
fn resolve_uses_provider_model_format_prior_when_exact_route_is_missing() {
    let source = route("provider-a", "key-a", "endpoint-a", "openai:chat", true);
    let target = route("provider-a", "key-b", "endpoint-b", "openai:chat", true);
    let catalog = catalog(vec![record(source, 30)]);

    let resolved = catalog.resolve(&target, fingerprints(), 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Prior);
    assert_eq!(resolved.prior_source, RoutingPriorSource::ProviderModelFormat);
    assert_eq!(resolved.prior_sample_count, 30);
}

#[test]
fn resolve_returns_neutral_prior_without_matching_metrics() {
    let catalog = catalog(Vec::new());
    let resolved = catalog.resolve(
        &route("provider-z", "key-z", "endpoint-z", "openai:chat", false),
        fingerprints(),
        20,
        RoutingMetricWindow::FiveMinutes,
    );

    assert_eq!(resolved.metric_source, RoutingMetricSource::Prior);
    assert_eq!(resolved.prior_source, RoutingPriorSource::Neutral);
    assert_eq!(resolved.prior_sample_count, 0);
}

fn catalog(records: Vec<RoutingMetricRecord>) -> MetricCatalog {
    let snapshot = RoutingMetricsSnapshot {
        windows: HashMap::from([(RoutingMetricWindow::FiveMinutes, records)]),
        route_states: Vec::new(),
        context_route_states: Vec::new(),
        refreshed_at: Some(time::OffsetDateTime::now_utc()),
    };
    MetricCatalog::from_snapshot(&snapshot, vec![RoutingMetricWindow::FiveMinutes])
}

fn record(route: RouteIdentity, sample_count: u64) -> RoutingMetricRecord {
    record_with_fingerprints(route, sample_count, CURRENT_ROUTE_FINGERPRINT, CURRENT_PRICE_FINGERPRINT)
}

fn record_with_fingerprints(route: RouteIdentity, sample_count: u64, route_fingerprint: &str, price_fingerprint: &str) -> RoutingMetricRecord {
    RoutingMetricRecord {
        route,
        provider_name: None,
        key_name: None,
        endpoint_name: None,
        route_config_fingerprint: Some(route_fingerprint.into()),
        price_config_fingerprint: Some(price_fingerprint.into()),
        snapshot: RoutingMetricSnapshot {
            request_count: sample_count,
            success_count: sample_count,
            sample_count,
            latency_avg_ms: Some(500.0),
            ttfb_avg_ms: Some(200.0),
            output_tps: Some(40.0),
            ..Default::default()
        },
        last_seen_at: time::OffsetDateTime::now_utc(),
    }
}

fn fingerprints() -> RouteFingerprints<'static> {
    RouteFingerprints {
        route_config: CURRENT_ROUTE_FINGERPRINT,
        price_config: CURRENT_PRICE_FINGERPRINT,
    }
}

fn route(provider_id: &str, key_id: &str, endpoint_id: &str, provider_api_format: &str, is_stream: bool) -> RouteIdentity {
    RouteIdentity {
        provider_id: provider_id.into(),
        key_id: key_id.into(),
        endpoint_id: endpoint_id.into(),
        global_model_id: "model-a".into(),
        client_api_format: "openai:chat".into(),
        provider_api_format: provider_api_format.into(),
        is_stream,
    }
}
