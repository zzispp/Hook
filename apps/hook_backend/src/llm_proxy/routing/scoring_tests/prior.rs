use types::provider::{RoutingMetricSource, RoutingMetricWindow, RoutingPriorSource};

use super::{candidate, component, make_normal, profile};
use crate::llm_proxy::routing::score_routes;

#[test]
fn prior_sample_cap_controls_effective_count_without_rewriting_raw_metrics() {
    let mut profile = profile();
    profile.prior_sample_cap = 5;
    let mut prior = candidate("key-prior", false);
    make_prior(&mut prior, 1_000);

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![prior]);
    let explanation = &scores[0].explanation;

    assert_eq!(explanation.state.as_str(), "warming");
    assert_eq!(explanation.raw_metrics.sample_count, 1_000);
    assert_eq!(explanation.prior_sample_count, 1_000);
    assert_eq!(explanation.effective_sample_count, 5);
}

#[test]
fn non_contextual_exploration_uses_effective_sample_count_for_prior_metrics() {
    let mut profile = profile();
    profile.contextual_exploration_enabled = false;
    profile.prior_sample_cap = 20;
    let mut prior = candidate("key-prior", false);
    make_prior(&mut prior, 1_000);
    let mut exact = candidate("key-exact", false);
    make_normal(&mut exact, 20, 20);

    let scores = score_routes(&profile, RoutingMetricWindow::FiveMinutes, vec![prior, exact]);
    let prior = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-prior")
        .expect("prior route should be scored");
    let exact = scores
        .iter()
        .find(|item| item.explanation.route.key_id == "key-exact")
        .expect("exact route should be scored");

    assert_eq!(prior.explanation.effective_sample_count, exact.explanation.effective_sample_count);
    assert_eq!(prior.explanation.effective_sample_count, 20);
    assert!((component(prior, "exploration").contribution - component(exact, "exploration").contribution).abs() < 0.000_1);
}

fn make_prior(candidate: &mut crate::llm_proxy::routing::RoutingScoreCandidate, sample_count: u64) {
    make_normal(candidate, sample_count, sample_count);
    candidate.metric_source = RoutingMetricSource::Prior;
    candidate.prior_source = RoutingPriorSource::Provider;
    candidate.prior_sample_count = sample_count;
}
