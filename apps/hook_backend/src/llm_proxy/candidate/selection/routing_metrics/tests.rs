use std::collections::HashMap;

use storage::provider::{RoutingContextRouteStateRecord, RoutingMetricRecord, RoutingRouteStateRecord};
use types::provider::{
    ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1, ROUTING_TIMING_SEMANTICS_LEGACY_FIRST_BYTE_V1, RouteIdentity, RoutingMetricSnapshot, RoutingMetricSource,
    RoutingMetricWindow, RoutingPriorSource, RoutingProfileId,
};

use super::{ContextRouteStateCatalog, MetricCatalog, RouteFingerprints, RouteStateCatalog, RoutingMetricsSnapshot};

const CURRENT_ROUTE_FINGERPRINT: &str = "route-fingerprint";
const CURRENT_PRICE_FINGERPRINT: &str = "price-fingerprint";

#[test]
fn resolve_prefers_exact_route_metric() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let catalog = catalog(vec![record(route.clone(), 25)]);

    let resolved = catalog.resolve(&route, fingerprints(), 20, 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Exact);
    assert_eq!(resolved.prior_source, RoutingPriorSource::ExactRoute);
    assert_eq!(resolved.prior_sample_count, 25);
}

#[test]
fn resolve_uses_prior_when_exact_route_fingerprint_mismatches() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let catalog = catalog(vec![record_with_fingerprints(route.clone(), 25, "old-route", "old-price")]);

    let resolved = catalog.resolve(&route, fingerprints(), 20, 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Prior);
    assert_eq!(resolved.prior_source, RoutingPriorSource::ProviderModelFormat);
    assert_eq!(resolved.prior_sample_count, 25);
    assert_eq!(resolved.effective_sample_count, 20);
}

#[test]
fn resolve_ignores_legacy_timing_semantics_in_prior_aggregation() {
    let source = route("provider-a", "key-a", "endpoint-a", "openai:chat", true);
    let target = route("provider-a", "key-b", "endpoint-b", "openai:chat", true);
    let catalog = catalog(vec![record_with_semantics(
        source,
        30,
        CURRENT_ROUTE_FINGERPRINT,
        CURRENT_PRICE_FINGERPRINT,
        ROUTING_TIMING_SEMANTICS_LEGACY_FIRST_BYTE_V1,
    )]);

    let resolved = catalog.resolve(&target, fingerprints(), 20, 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Prior);
    assert_eq!(resolved.prior_source, RoutingPriorSource::Neutral);
    assert_eq!(resolved.prior_sample_count, 0);
}

#[test]
fn resolve_keeps_multiple_fingerprint_versions_for_same_route() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let catalog = catalog(vec![
        record_with_fingerprints(route.clone(), 10, "old-route", "old-price"),
        record(route.clone(), 25),
    ]);

    let resolved = catalog.resolve(&route, fingerprints(), 20, 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Exact);
    assert_eq!(resolved.prior_sample_count, 25);
}

#[test]
fn resolve_uses_provider_model_format_prior_when_exact_route_is_missing() {
    let source = route("provider-a", "key-a", "endpoint-a", "openai:chat", true);
    let target = route("provider-a", "key-b", "endpoint-b", "openai:chat", true);
    let catalog = catalog(vec![record(source, 30)]);

    let resolved = catalog.resolve(&target, fingerprints(), 20, 20, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.metric_source, RoutingMetricSource::Prior);
    assert_eq!(resolved.prior_source, RoutingPriorSource::ProviderModelFormat);
    assert_eq!(resolved.prior_sample_count, 30);
    assert_eq!(resolved.effective_sample_count, 20);
}

#[test]
fn resolve_returns_neutral_prior_without_matching_metrics() {
    let catalog = catalog(Vec::new());
    let resolved = catalog.resolve(
        &route("provider-z", "key-z", "endpoint-z", "openai:chat", false),
        fingerprints(),
        20,
        20,
        RoutingMetricWindow::FiveMinutes,
    );

    assert_eq!(resolved.metric_source, RoutingMetricSource::Prior);
    assert_eq!(resolved.prior_source, RoutingPriorSource::Neutral);
    assert_eq!(resolved.prior_sample_count, 0);
}

#[test]
fn resolve_caps_prior_sample_count_for_scoring() {
    let source = route("provider-a", "key-a", "endpoint-a", "openai:chat", true);
    let target = route("provider-a", "key-b", "endpoint-b", "openai:chat", true);
    let catalog = catalog(vec![record(source, 30)]);

    let resolved = catalog.resolve(&target, fingerprints(), 20, 5, RoutingMetricWindow::FiveMinutes);

    assert_eq!(resolved.prior_sample_count, 30);
    assert_eq!(resolved.effective_sample_count, 5);
}

#[test]
fn route_state_catalog_matches_profile_scoped_ema() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let snapshot = RoutingMetricsSnapshot {
        windows: HashMap::new(),
        route_states: vec![
            route_state_record(RoutingProfileId::Balanced, route.clone(), 0.91),
            route_state_record(RoutingProfileId::HighAvailability, route.clone(), 0.42),
        ],
        context_route_states: Vec::new(),
        refreshed_at: Some(time::OffsetDateTime::now_utc()),
    };

    let catalog = RouteStateCatalog::from_snapshot(&snapshot);
    let balanced = catalog
        .record(RoutingProfileId::Balanced, &route, fingerprints())
        .expect("balanced ema should exist");
    let ha = catalog
        .record(RoutingProfileId::HighAvailability, &route, fingerprints())
        .expect("ha ema should exist");

    assert_eq!(balanced.success_rate, 0.91);
    assert_eq!(ha.success_rate, 0.42);
}

#[test]
fn context_state_catalog_counts_only_matching_profile() {
    let route = route("provider-a", "key-a", "endpoint-a", "openai:chat", false);
    let snapshot = RoutingMetricsSnapshot {
        windows: HashMap::new(),
        route_states: Vec::new(),
        context_route_states: vec![
            context_state_record(RoutingProfileId::Balanced, "ctx", route.clone(), 7),
            context_state_record(RoutingProfileId::HighAvailability, "ctx", route.clone(), 19),
        ],
        refreshed_at: Some(time::OffsetDateTime::now_utc()),
    };

    let catalog = ContextRouteStateCatalog::from_snapshot(&snapshot);
    let balanced = catalog.samples(RoutingProfileId::Balanced, "ctx", &route, fingerprints());
    let ha = catalog.samples(RoutingProfileId::HighAvailability, "ctx", &route, fingerprints());

    assert_eq!(balanced.route_sample_count, 7);
    assert_eq!(balanced.total_sample_count, 7);
    assert_eq!(ha.route_sample_count, 19);
    assert_eq!(ha.total_sample_count, 19);
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
    record_with_semantics(
        route,
        sample_count,
        CURRENT_ROUTE_FINGERPRINT,
        CURRENT_PRICE_FINGERPRINT,
        ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1,
    )
}

fn record_with_fingerprints(route: RouteIdentity, sample_count: u64, route_fingerprint: &str, price_fingerprint: &str) -> RoutingMetricRecord {
    record_with_semantics(
        route,
        sample_count,
        route_fingerprint,
        price_fingerprint,
        ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1,
    )
}

fn record_with_semantics(
    route: RouteIdentity,
    sample_count: u64,
    route_fingerprint: &str,
    price_fingerprint: &str,
    timing_metric_semantics_version: &str,
) -> RoutingMetricRecord {
    RoutingMetricRecord {
        route,
        provider_name: None,
        key_name: None,
        endpoint_name: None,
        timing_metric_semantics_version: timing_metric_semantics_version.into(),
        route_config_fingerprint: Some(route_fingerprint.into()),
        price_config_fingerprint: Some(price_fingerprint.into()),
        snapshot: RoutingMetricSnapshot {
            request_count: sample_count,
            success_count: sample_count,
            sample_count,
            latency_avg_ms: Some(500.0),
            first_token_avg_ms: Some(200.0),
            output_tps: Some(40.0),
            ..Default::default()
        },
        last_seen_at: time::OffsetDateTime::now_utc(),
    }
}

fn route_state_record(profile_id: RoutingProfileId, route: RouteIdentity, success_rate: f64) -> RoutingRouteStateRecord {
    RoutingRouteStateRecord {
        profile_id: profile_id.as_str().to_owned(),
        route,
        timing_metric_semantics_version: ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1.into(),
        ema_success_rate: success_rate,
        ema_first_token_ms: Some(100.0),
        ema_latency_ms: Some(300.0),
        ema_output_tps: Some(50.0),
        sample_count: 12,
        route_config_fingerprint: Some(CURRENT_ROUTE_FINGERPRINT.into()),
        price_config_fingerprint: Some(CURRENT_PRICE_FINGERPRINT.into()),
        last_updated_at: time::OffsetDateTime::now_utc(),
    }
}

fn context_state_record(profile_id: RoutingProfileId, context_key: &str, route: RouteIdentity, sample_count: u64) -> RoutingContextRouteStateRecord {
    RoutingContextRouteStateRecord {
        profile_id: profile_id.as_str().to_owned(),
        context_key: context_key.into(),
        route,
        timing_metric_semantics_version: ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1.into(),
        sample_count,
        success_count: sample_count,
        failure_count: 0,
        ema_success_rate: 1.0,
        ema_first_token_ms: Some(100.0),
        ema_latency_ms: Some(300.0),
        ema_output_tps: Some(50.0),
        route_config_fingerprint: Some(CURRENT_ROUTE_FINGERPRINT.into()),
        price_config_fingerprint: Some(CURRENT_PRICE_FINGERPRINT.into()),
        last_updated_at: time::OffsetDateTime::now_utc(),
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
