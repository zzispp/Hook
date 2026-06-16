use types::provider::{
    RouteIdentity, RoutingMetricSnapshot, RoutingMetricSource, RoutingMetricWindow, RoutingPriorSource, RoutingProfile, RoutingProfileId, RoutingProfileWeights,
};

use super::{RoutingEmaSnapshot, RoutingScoreCandidate, circuit::CircuitCandidateState, score_routes};

mod exploration;

#[test]
fn warming_candidates_keep_cache_affinity_bonus() {
    let profile = profile();
    let uncached = candidate("key-a", false);
    let cached = candidate("key-b", true);

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![uncached, cached]);

    assert_eq!(scores[0].explanation.route.key_id, "key-b");
    assert!(
        scores[0].explanation.final_score > scores[1].explanation.final_score,
        "cached warming route should outrank the uncached route"
    );
}

#[test]
fn recent_regression_degrades_fallback_metrics() {
    let profile = profile();
    let stable = candidate("key-stable", false);
    let mut regressed = candidate("key-regressed", false);
    regressed.metric_window = RoutingMetricWindow::OneDay;
    regressed.metric.sample_count = 120;
    regressed.metric.success_count = 118;
    regressed.metric.request_count = 120;
    regressed.metric.latency_avg_ms = Some(420.0);
    regressed.metric.ttfb_avg_ms = Some(180.0);
    regressed.recent_metric = Some(RoutingMetricSnapshot {
        request_count: 5,
        success_count: 2,
        failure_count: 3,
        sample_count: 5,
        latency_avg_ms: Some(2_400.0),
        ttfb_avg_ms: Some(1_400.0),
        ..Default::default()
    });

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![stable, regressed]);
    let regressed = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-regressed")
        .expect("regressed route should be present");

    assert_eq!(regressed.explanation.state.as_str(), "degraded");
    assert!(
        regressed
            .explanation
            .components
            .iter()
            .any(|component| component.code == "recent_regression" && component.contribution < 0.0)
    );
    assert!(!regressed.explanation.components.iter().any(|component| component.code == "exploration"));
}

#[test]
fn ema_recent_penalizes_worse_recent_state_with_cap() {
    let profile = profile();
    let mut candidate = candidate("key-ema-worse", false);
    make_normal(&mut candidate, 40, 40);
    candidate.ema = Some(RoutingEmaSnapshot {
        success_rate: 0.20,
        ttfb_avg_ms: Some(4_000.0),
        latency_avg_ms: Some(12_000.0),
        output_tps: Some(5.0),
        sample_count: 40,
        freshness_seconds: 60,
    });

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![candidate]);
    let component = component(&scores[0], "ema_recent");

    assert!(component.contribution < 0.0);
    assert!(component.contribution >= -6.0);
}

#[test]
fn ema_recent_rewards_better_recent_state_with_cap() {
    let profile = profile();
    let mut candidate = candidate("key-ema-better", false);
    make_normal(&mut candidate, 30, 40);
    candidate.metric.latency_avg_ms = Some(6_000.0);
    candidate.metric.ttfb_avg_ms = Some(2_000.0);
    candidate.metric.output_tps = Some(8.0);
    candidate.ema = Some(RoutingEmaSnapshot {
        success_rate: 1.0,
        ttfb_avg_ms: Some(150.0),
        latency_avg_ms: Some(300.0),
        output_tps: Some(120.0),
        sample_count: 40,
        freshness_seconds: 60,
    });

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![candidate]);
    let component = component(&scores[0], "ema_recent");

    assert!(component.contribution > 0.0);
    assert!(component.contribution <= 6.0);
}

#[test]
fn balanced_profile_ignores_priority_contribution() {
    let profile = profile();
    let mut low_priority = candidate("key-low-priority", false);
    low_priority.admin_priority = 1;
    low_priority.metric.sample_count = 40;
    low_priority.metric.request_count = 40;
    low_priority.metric.success_count = 40;
    let mut high_priority = candidate("key-high-priority", false);
    high_priority.admin_priority = 900;
    high_priority.metric.sample_count = 40;
    high_priority.metric.request_count = 40;
    high_priority.metric.success_count = 40;

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![low_priority, high_priority]);
    let low = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-low-priority")
        .expect("low priority route should exist");
    let high = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-high-priority")
        .expect("high priority route should exist");

    assert_eq!(low.explanation.final_score, high.explanation.final_score);
    assert!(
        low.explanation
            .components
            .iter()
            .any(|component| component.code == "priority" && component.contribution == 0.0)
    );
    assert!(
        high.explanation
            .components
            .iter()
            .any(|component| component.code == "priority" && component.contribution == 0.0)
    );
}

#[test]
fn warming_balanced_profile_ignores_priority_contribution() {
    let profile = profile();
    let mut low_priority = candidate("key-warm-low", false);
    low_priority.admin_priority = 1;
    let mut high_priority = candidate("key-warm-high", false);
    high_priority.admin_priority = 900;

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![low_priority, high_priority]);
    let low = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-warm-low")
        .expect("low priority route should exist");
    let high = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-warm-high")
        .expect("high priority route should exist");

    assert_eq!(low.explanation.final_score, high.explanation.final_score);
    assert!(
        low.explanation
            .components
            .iter()
            .any(|component| component.code == "priority" && component.contribution == 0.0)
    );
    assert!(
        high.explanation
            .components
            .iter()
            .any(|component| component.code == "priority" && component.contribution == 0.0)
    );
}

#[test]
fn balanced_profile_ignores_priority_even_if_overlay_weight_is_dirty() {
    let mut profile = profile();
    profile.weights.priority = 0.44;
    let mut low_priority = candidate("key-dirty-low", false);
    low_priority.admin_priority = 1;
    let mut high_priority = candidate("key-dirty-high", false);
    high_priority.admin_priority = 900;

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![low_priority, high_priority]);
    let low = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-dirty-low")
        .expect("low priority route should exist");
    let high = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-dirty-high")
        .expect("high priority route should exist");

    assert_eq!(low.explanation.final_score, high.explanation.final_score);
    assert!(
        low.explanation
            .components
            .iter()
            .any(|component| component.code == "priority" && component.contribution == 0.0)
    );
    assert!(
        high.explanation
            .components
            .iter()
            .any(|component| component.code == "priority" && component.contribution == 0.0)
    );
}

fn candidate(key_id: &str, is_cached: bool) -> RoutingScoreCandidate {
    RoutingScoreCandidate {
        route: RouteIdentity {
            provider_id: "provider-a".into(),
            key_id: key_id.into(),
            endpoint_id: "endpoint-a".into(),
            global_model_id: "model-a".into(),
            client_api_format: "openai:chat".into(),
            provider_api_format: "openai:chat".into(),
            is_stream: false,
        },
        provider_name: Some("Provider A".into()),
        key_name: Some(key_id.into()),
        key_preview: Some("sk-***".into()),
        endpoint_name: Some("openai:chat".into()),
        metric: RoutingMetricSnapshot {
            request_count: 8,
            success_count: 8,
            sample_count: 8,
            latency_avg_ms: Some(500.0),
            ttfb_avg_ms: Some(200.0),
            output_tps: Some(40.0),
            ..Default::default()
        },
        metric_window: RoutingMetricWindow::FiveMinutes,
        metric_freshness_seconds: 15,
        recent_metric: None,
        metric_source: RoutingMetricSource::Exact,
        prior_source: RoutingPriorSource::ExactRoute,
        prior_sample_count: 8,
        routing_context_key: Some("group=default|model=model-a|format=openai:chat|stream=false|size=unknown|cap=none".into()),
        route_config_fingerprint: Some("route-fingerprint".into()),
        price_config_fingerprint: Some("price-fingerprint".into()),
        context_route_sample_count: 8,
        context_total_sample_count: 8,
        ema: None,
        circuit_state: CircuitCandidateState::Closed,
        admin_priority: 10,
        estimated_cost: None,
        needs_conversion: false,
        is_cached,
        request_features: types::provider::RoutingRequestFeatures::unknown("openai:chat", false, None),
    }
}

fn make_normal(candidate: &mut RoutingScoreCandidate, success_count: u64, request_count: u64) {
    candidate.metric.sample_count = request_count;
    candidate.metric.request_count = request_count;
    candidate.metric.success_count = success_count;
    candidate.context_route_sample_count = request_count;
    candidate.context_total_sample_count = request_count;
}

fn component<'a>(score: &'a super::ScoredRoute, code: &str) -> &'a types::provider::ScoreComponent {
    score
        .explanation
        .components
        .iter()
        .find(|component| component.code == code)
        .expect("score component should exist")
}

fn profile() -> RoutingProfile {
    RoutingProfile {
        id: RoutingProfileId::Balanced,
        name: "Balanced".into(),
        description: "test".into(),
        weights: RoutingProfileWeights {
            success: 0.28,
            ttfb: 0.19,
            latency: 0.17,
            tps: 0.09,
            cost: 0.15,
            headroom: 0.12,
            priority: 0.0,
        },
        version: "test".into(),
        min_samples: 20,
        exploration_k: 3.0,
        conversion_penalty: 6.0,
        stale_metric_penalty: 8.0,
        affinity_bonus: 6.0,
        prior_sample_cap: types::provider::default_prior_sample_cap(),
        contextual_exploration_enabled: types::provider::default_contextual_exploration_enabled(),
        auto_tune_enabled: false,
        learning: None,
    }
}
