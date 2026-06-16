use storage::provider::RoutingProfileVersionSnapshot;
use time::{Duration, OffsetDateTime};
use types::provider::{RoutingMetricWindow, RoutingProfileId, RoutingProfileWeights};

use super::learning::{blend_weights, needs_refresh};

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
fn zero_sample_snapshot_does_not_force_refresh_before_interval() {
    let profile = super::profiles::test_only_builtin_profile(RoutingProfileId::Balanced);
    let now = OffsetDateTime::now_utc();
    let snapshot = snapshot(&profile.weights, 0, now - Duration::minutes(5));

    assert!(!needs_refresh(Some(&snapshot), &profile, now));
}

#[test]
fn zero_sample_admin_profile_version_refreshes_immediately() {
    let mut profile = super::profiles::test_only_builtin_profile(RoutingProfileId::Balanced);
    profile.version = "profile-admin-save".into();
    let now = OffsetDateTime::now_utc();
    let mut snapshot = snapshot(&profile.weights, 0, now - Duration::minutes(5));
    snapshot.profile_version = profile.version.clone();

    assert!(needs_refresh(Some(&snapshot), &profile, now));
}

#[test]
fn learning_refreshes_when_admin_weights_change() {
    let mut profile = super::profiles::test_only_builtin_profile(RoutingProfileId::Balanced);
    let now = OffsetDateTime::now_utc();
    let snapshot = snapshot(&profile.weights, 0, now - Duration::minutes(5));
    profile.weights.cost += 0.01;
    profile.weights.success -= 0.01;

    assert!(needs_refresh(Some(&snapshot), &profile, now));
}

#[test]
fn learning_refreshes_after_interval() {
    let profile = super::profiles::test_only_builtin_profile(RoutingProfileId::Balanced);
    let now = OffsetDateTime::now_utc();
    let snapshot = snapshot(&profile.weights, 0, now - Duration::minutes(16));

    assert!(needs_refresh(Some(&snapshot), &profile, now));
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

fn snapshot(admin_weights: &RoutingProfileWeights, sample_count: u64, created_at: OffsetDateTime) -> RoutingProfileVersionSnapshot {
    RoutingProfileVersionSnapshot {
        profile_id: "balanced".into(),
        profile_version: "test".into(),
        admin_weights: admin_weights.clone(),
        learned_weights: None,
        effective_weights: admin_weights.clone(),
        reward_window: RoutingMetricWindow::SevenDays,
        sample_count,
        created_at,
    }
}
