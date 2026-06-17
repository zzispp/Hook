use types::provider::RoutingMetricWindow;

use super::{candidate, component, make_normal, profile};
use crate::llm_proxy::routing::score_routes;

#[test]
fn normal_candidates_receive_capped_exploration_bonus() {
    let profile = profile();
    let mut explored = candidate("key-explored", false);
    make_normal(&mut explored, 40, 40);

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![explored]);
    let component = component(&scores[0], "exploration");

    assert!(component.contribution > 0.0);
    assert!(component.contribution <= profile.exploration_cap);
}

#[test]
fn normal_candidates_use_profile_exploration_weight_and_cap() {
    let mut profile = profile();
    profile.exploration_weight = 0.5;
    profile.exploration_cap = 1.2;
    let mut explored = candidate("key-exploration-profile", false);
    make_normal(&mut explored, 40, 40);

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![explored]);
    let component = component(&scores[0], "exploration");

    assert_eq!(component.weight, 0.5);
    assert!(component.contribution <= 1.2);
}

#[test]
fn contextual_exploration_rewards_low_context_samples() {
    let profile = profile();
    let mut low_context = candidate("key-context-low", false);
    make_normal(&mut low_context, 40, 40);
    low_context.context_route_sample_count = 1;
    low_context.context_total_sample_count = 41;
    let mut high_context = candidate("key-context-high", false);
    make_normal(&mut high_context, 40, 40);
    high_context.context_route_sample_count = 40;
    high_context.context_total_sample_count = 41;

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![low_context, high_context]);
    let low = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-context-low")
        .expect("low context route should exist");
    let high = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-context-high")
        .expect("high context route should exist");

    assert!(component(low, "exploration").contribution > component(high, "exploration").contribution);
}

#[test]
fn degraded_normal_candidates_do_not_receive_exploration_bonus() {
    let profile = profile();
    let mut stale = candidate("key-stale", false);
    make_normal(&mut stale, 40, 40);
    stale.metric_freshness_seconds = 1_000;

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![stale]);

    assert_eq!(scores[0].explanation.state.as_str(), "degraded");
    assert!(!scores[0].explanation.components.iter().any(|component| component.code == "exploration"));
}

#[test]
fn exploration_uses_profile_success_threshold() {
    let mut profile = profile();
    profile.exploration_min_success_score = 65.0;
    let mut candidate = candidate("key-exploration-threshold", false);
    make_normal(&mut candidate, 70, 100);

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![candidate]);

    assert!(scores[0].explanation.components.iter().any(|component| component.code == "exploration"));
}
