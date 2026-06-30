use storage::provider::RoutingMetricRecord;
use types::provider::{ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1, RouteIdentity, RoutingMetricSnapshot, RoutingProfileId, RoutingProfileWeights};

use super::learning::{blend_weights, quality_score};

#[test]
fn blend_weights_keeps_fixed_priority_weight_locked() {
    let admin = weights(0.18, 0.08, 0.08, 0.04, 0.08, 0.10, 0.44);
    let learned = weights(0.40, 0.22, 0.12, 0.10, 0.16, 0.0, 0.0);

    let effective = blend_weights(&admin, &learned, 0.35);

    assert!((effective.priority - admin.priority).abs() < 1e-9);
    assert!((effective.headroom - admin.headroom).abs() < 1e-9);
    assert!((sum(&effective) - 1.0).abs() < 1e-9);
}

#[test]
fn blend_weights_keeps_non_fixed_headroom_locked_and_priority_zero() {
    let admin = built_in_balanced_weights();
    let learned = weights(0.36, 0.12, 0.12, 0.18, 0.22, 0.0, 0.30);

    let effective = blend_weights(&admin, &learned, 0.35);

    assert!((effective.headroom - admin.headroom).abs() < 1e-9);
    assert_eq!(effective.priority, 0.0);
    assert!((sum(&effective) - 1.0).abs() < 1e-9);
}

#[test]
fn builtin_profiles_enable_auto_tune_by_default() {
    for id in [
        RoutingProfileId::Balanced,
        RoutingProfileId::FirstByte,
        RoutingProfileId::HighTps,
        RoutingProfileId::CostOptimal,
        RoutingProfileId::HighAvailability,
        RoutingProfileId::CacheAffinityPlus,
        RoutingProfileId::FixedPriorityPlus,
        RoutingProfileId::Custom,
    ] {
        assert!(super::profiles::test_only_builtin_profile(id).auto_tune_enabled);
    }
}

#[test]
fn quality_score_penalizes_observable_quality_failures() {
    let clean = metric(20, 0);
    let degraded = metric(20, 5);

    assert_eq!(quality_score(&clean), 1.0);
    assert!(quality_score(&degraded) < quality_score(&clean));
    assert_eq!(quality_score(&degraded), 0.75);
}

fn built_in_balanced_weights() -> RoutingProfileWeights {
    weights(0.28, 0.19, 0.17, 0.09, 0.15, 0.12, 0.0)
}

fn metric(request_count: u64, quality_failures: u64) -> RoutingMetricRecord {
    RoutingMetricRecord {
        route: RouteIdentity {
            provider_id: "provider-a".into(),
            key_id: "key-a".into(),
            endpoint_id: "endpoint-a".into(),
            global_model_id: "model-a".into(),
            client_api_format: "openai:chat".into(),
            provider_api_format: "openai:chat".into(),
            is_stream: false,
        },
        provider_name: None,
        key_name: None,
        endpoint_name: None,
        timing_metric_semantics_version: ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1.into(),
        route_config_fingerprint: Some("route-fingerprint".into()),
        price_config_fingerprint: Some("price-fingerprint".into()),
        snapshot: RoutingMetricSnapshot {
            request_count,
            success_count: request_count,
            sample_count: request_count,
            format_conversion_failure_count: quality_failures,
            ..Default::default()
        },
        last_seen_at: time::OffsetDateTime::now_utc(),
    }
}

fn weights(success: f64, ttfb: f64, latency: f64, tps: f64, cost: f64, headroom: f64, priority: f64) -> RoutingProfileWeights {
    RoutingProfileWeights {
        success,
        ttfb,
        latency,
        tps,
        cost,
        headroom,
        priority,
    }
}

fn sum(weights: &RoutingProfileWeights) -> f64 {
    weights.success + weights.ttfb + weights.latency + weights.tps + weights.cost + weights.headroom + weights.priority
}
