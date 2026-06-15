use types::provider::{RoutingProfileId, RoutingProfileWeights};

use super::learning::blend_weights;

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

fn built_in_balanced_weights() -> RoutingProfileWeights {
    weights(0.28, 0.19, 0.17, 0.09, 0.15, 0.12, 0.0)
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
