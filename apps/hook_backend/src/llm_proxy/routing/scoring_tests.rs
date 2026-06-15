use types::provider::{
    RouteIdentity, RoutingCacheAffinityMode, RoutingMetricSnapshot, RoutingMetricWindow, RoutingProfile, RoutingProfileId, RoutingProfileWeights,
};

use super::{RoutingEmaSnapshot, RoutingScoreCandidate, ScoreRoutesInput, ScoredRoute, circuit::CircuitCandidateState, score_routes};

const TEST_REQUEST_ID: &str = "routing-score-test";

#[test]
fn warming_candidates_keep_cache_affinity_bonus() {
    let profile = profile();
    let uncached = candidate("key-a", false);
    let cached = candidate("key-b", true);

    let scores = score(&profile, vec![uncached, cached]);

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

    let scores = score(&profile, vec![stable, regressed]);
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

    let scores = score(&profile, vec![low_priority, high_priority]);
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

    let scores = score(&profile, vec![low_priority, high_priority]);
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

    let scores = score(&profile, vec![low_priority, high_priority]);
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

#[test]
fn ema_regression_penalizes_worse_recent_state() {
    let profile = profile();
    let mut route = normal_candidate("key-ema-worse");
    route.ema = Some(RoutingEmaSnapshot {
        success_rate: 0.50,
        latency_ms: Some(3_000.0),
        ttfb_ms: Some(1_800.0),
        sample_count: 80,
    });

    let scores = score(&profile, vec![route]);
    let component = scores[0]
        .explanation
        .components
        .iter()
        .find(|component| component.code == "ema_regression")
        .expect("worse EMA should add regression component");

    assert!(component.contribution < 0.0);
}

#[test]
fn ema_improvement_does_not_add_positive_bonus() {
    let profile = profile();
    let mut route = normal_candidate("key-ema-better");
    route.ema = Some(RoutingEmaSnapshot {
        success_rate: 1.0,
        latency_ms: Some(200.0),
        ttfb_ms: Some(80.0),
        sample_count: 80,
    });

    let scores = score(&profile, vec![route]);

    assert!(!scores[0].explanation.components.iter().any(|component| component.code == "ema_regression"));
}

#[test]
fn low_sample_ema_state_is_ignored() {
    let profile = profile();
    let mut route = normal_candidate("key-ema-low-sample");
    route.ema = Some(RoutingEmaSnapshot {
        success_rate: 0.10,
        latency_ms: Some(10_000.0),
        ttfb_ms: Some(3_000.0),
        sample_count: 3,
    });

    let scores = score(&profile, vec![route]);

    assert!(!scores[0].explanation.components.iter().any(|component| component.code == "ema_regression"));
}

#[test]
fn exploration_budget_zero_disables_warming_ucb_contribution() {
    let mut profile = profile();
    profile.exploration_budget_percent = 0.0;

    let scores = score_with_request(&profile, "request-no-explore", vec![candidate("key-no-explore", false)]);
    let exploration = exploration_component(&scores[0]);

    assert_eq!(exploration.weight, 0.0);
    assert_eq!(exploration.contribution, 0.0);
    assert!(exploration.normalized_score > 0.0);
}

#[test]
fn exploration_budget_full_preserves_warming_ucb_contribution() {
    let mut profile = profile();
    profile.exploration_budget_percent = 100.0;

    let scores = score_with_request(&profile, "request-explore", vec![candidate("key-explore", false)]);
    let exploration = exploration_component(&scores[0]);

    assert!(exploration.weight > 0.0);
    assert!(exploration.contribution > 0.0);
}

#[test]
fn exploration_budget_gate_is_deterministic_for_request_id() {
    let mut profile = profile();
    profile.exploration_budget_percent = 10.0;
    let route = candidate("key-stable-explore", false);

    let first = score_with_request(&profile, "stable-request", vec![route.clone()]);
    let second = score_with_request(&profile, "stable-request", vec![route]);

    assert_eq!(first[0].explanation.final_score, second[0].explanation.final_score);
    assert_eq!(exploration_component(&first[0]).contribution, exploration_component(&second[0]).contribution);
}

fn score(profile: &RoutingProfile, candidates: Vec<RoutingScoreCandidate>) -> Vec<ScoredRoute> {
    score_with_request(profile, TEST_REQUEST_ID, candidates)
}

fn score_with_request(profile: &RoutingProfile, request_id: &str, candidates: Vec<RoutingScoreCandidate>) -> Vec<ScoredRoute> {
    score_routes(ScoreRoutesInput {
        profile,
        window: RoutingMetricWindow::FiveMinutes,
        request_id,
        candidates,
    })
}

fn exploration_component(score: &ScoredRoute) -> &types::provider::ScoreComponent {
    score
        .explanation
        .components
        .iter()
        .find(|component| component.code == "exploration")
        .expect("warming route should include exploration component")
}

fn normal_candidate(key_id: &str) -> RoutingScoreCandidate {
    let mut output = candidate(key_id, false);
    output.metric.request_count = 80;
    output.metric.success_count = 76;
    output.metric.sample_count = 80;
    output.metric.latency_avg_ms = Some(500.0);
    output.metric.ttfb_avg_ms = Some(200.0);
    output
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
        ema: None,
        circuit_state: CircuitCandidateState::Closed,
        admin_priority: 10,
        estimated_cost: None,
        needs_conversion: false,
        affinity_bonus: is_cached,
    }
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
        exploration_budget_percent: 100.0,
        conversion_penalty: 6.0,
        stale_metric_penalty: 8.0,
        ema_regression_penalty: 6.0,
        cache_affinity_mode: RoutingCacheAffinityMode::ScoreBonus,
        affinity_bonus: 6.0,
        auto_tune_enabled: false,
        learning: None,
    }
}
