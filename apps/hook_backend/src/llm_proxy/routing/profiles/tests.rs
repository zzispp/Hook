use types::provider::{RoutingProfileId, RoutingProfileUpsert, RoutingProfileWeights};

use crate::llm_proxy::LlmProxyError;

use super::validate_profile_upsert;

#[test]
fn rejects_zero_min_samples() {
    let patch = RoutingProfileUpsert {
        min_samples: Some(0),
        ..Default::default()
    };

    assert_invalid(RoutingProfileId::Balanced, patch, "min_samples");
}

#[test]
fn rejects_negative_weight() {
    let patch = RoutingProfileUpsert {
        weights: Some(weights(-0.1, 0.29, 0.17, 0.09, 0.15, 0.12, 0.0)),
        ..Default::default()
    };

    assert_invalid(RoutingProfileId::Balanced, patch, "success");
}

#[test]
fn rejects_weight_sum_drift() {
    let patch = RoutingProfileUpsert {
        weights: Some(weights(0.30, 0.19, 0.17, 0.09, 0.15, 0.12, 0.0)),
        ..Default::default()
    };

    assert_invalid(RoutingProfileId::Balanced, patch, "sum to 1");
}

#[test]
fn rejects_priority_weight_outside_fixed_priority_profile() {
    let patch = RoutingProfileUpsert {
        weights: Some(weights(0.28, 0.19, 0.17, 0.09, 0.05, 0.12, 0.10)),
        ..Default::default()
    };

    assert_invalid(RoutingProfileId::Balanced, patch, "fixed_priority_plus");
}

#[test]
fn fixed_priority_profile_accepts_priority_weight() {
    let patch = RoutingProfileUpsert {
        weights: Some(weights(0.18, 0.08, 0.08, 0.04, 0.08, 0.10, 0.44)),
        ..Default::default()
    };

    assert!(validate_profile_upsert(RoutingProfileId::FixedPriorityPlus, &patch).is_ok());
}

#[test]
fn rejects_negative_exploration_k() {
    let patch = RoutingProfileUpsert {
        exploration_k: Some(-0.1),
        ..Default::default()
    };

    assert_invalid(RoutingProfileId::Balanced, patch, "exploration_k");
}

#[test]
fn rejects_negative_penalty_and_bonus_fields() {
    assert_invalid(
        RoutingProfileId::Balanced,
        RoutingProfileUpsert {
            conversion_penalty: Some(-0.1),
            ..Default::default()
        },
        "conversion_penalty",
    );
    assert_invalid(
        RoutingProfileId::Balanced,
        RoutingProfileUpsert {
            stale_metric_penalty: Some(-0.1),
            ..Default::default()
        },
        "stale_metric_penalty",
    );
    assert_invalid(
        RoutingProfileId::Balanced,
        RoutingProfileUpsert {
            affinity_bonus: Some(-0.1),
            ..Default::default()
        },
        "affinity_bonus",
    );
}

fn assert_invalid(id: RoutingProfileId, patch: RoutingProfileUpsert, expected: &str) {
    let error = validate_profile_upsert(id, &patch).expect_err("patch should be invalid");
    assert!(
        matches!(&error, LlmProxyError::InvalidRequest(message) if message.contains(expected)),
        "unexpected error: {error}"
    );
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
