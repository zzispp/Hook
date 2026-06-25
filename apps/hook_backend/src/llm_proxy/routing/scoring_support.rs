use types::provider::{RoutingMetricSnapshot, RoutingMetricWindow, RoutingProfile, RoutingProfileId, RoutingProfileWeights, RoutingRouteState, ScoreComponent};

use super::{
    scoring::math::{clamp_score, exploration_score, higher_is_better, lower_is_better, priority_score, soft_degraded, stale, success_score},
    scoring::{LATENCY_BAD_MS, LATENCY_GOOD_MS, RoutingEmaSnapshot, RoutingScoreCandidate, TPS_HIGH, TPS_LOW, TTFB_BAD_MS, TTFB_GOOD_MS},
};

pub(super) fn normal_adjustment_components(
    profile: &RoutingProfile,
    window: RoutingMetricWindow,
    total_attempts: u64,
    candidate: &RoutingScoreCandidate,
    recent_penalty: f64,
) -> Vec<ScoreComponent> {
    let mut output = Vec::new();
    if let Some(component) = ema_recent_component(profile, candidate) {
        output.push(component);
    }
    if let Some(component) = normal_exploration_component(profile, window, total_attempts, candidate, recent_penalty) {
        output.push(component);
    }
    output
}

pub(super) fn penalty_components(
    profile: &RoutingProfile,
    window: RoutingMetricWindow,
    candidate: &RoutingScoreCandidate,
    recent_penalty: f64,
) -> Vec<ScoreComponent> {
    let mut output = Vec::new();
    if candidate.is_cached {
        output.push(additive_component("affinity", "cache affinity", profile.affinity_bonus));
    }
    if candidate.needs_conversion {
        output.push(additive_component("conversion", "format conversion", -profile.conversion_penalty));
    }
    if stale(candidate, window) {
        output.push(additive_component("stale", "stale metrics", -profile.stale_metric_penalty));
    }
    if recent_penalty > 0.0 {
        output.push(additive_component("recent_regression", "recent window regression", -recent_penalty));
    }
    output
}

pub(super) fn soft_state(candidate: &RoutingScoreCandidate, window: RoutingMetricWindow, recent_penalty: f64) -> RoutingRouteState {
    if recent_penalty > 0.0 || soft_degraded(candidate, window) {
        return RoutingRouteState::Degraded;
    }
    RoutingRouteState::Eligible
}

pub(super) fn recent_regression_penalty(profile: &RoutingProfile, candidate: &RoutingScoreCandidate, window: RoutingMetricWindow) -> f64 {
    if candidate.metric_window == window {
        return 0.0;
    }
    let Some(recent) = candidate.recent_metric.as_ref() else {
        return 0.0;
    };
    if recent.sample_count == 0 {
        return 0.0;
    }
    let success_gap = (success_score(&candidate.metric) - success_score(recent)).max(0.0) / 100.0;
    let latency_gap = gap_ratio(recent.latency_avg_ms, candidate.metric.latency_avg_ms, LATENCY_BAD_MS);
    let ttfb_gap = gap_ratio(recent.ttfb_avg_ms, candidate.metric.ttfb_avg_ms, TTFB_BAD_MS);
    clamp_score((success_gap * 0.6 + latency_gap * 0.2 + ttfb_gap * 0.2) * profile.stale_metric_penalty)
}

pub(super) fn selected_reason(score: f64, components: &[ScoreComponent]) -> String {
    let parts = components
        .iter()
        .filter(|component| component.contribution.abs() > 0.01)
        .map(|component| format!("{} {:+.1}", component.label, component.contribution))
        .collect::<Vec<_>>()
        .join(", ");
    format!("score {score:.1}: {parts}")
}

pub(super) fn priority_component(profile: &RoutingProfile, priority: i32) -> ScoreComponent {
    let weight = effective_priority_weight(profile);
    if weight <= f64::EPSILON {
        return component("priority", "admin priority", Some(priority as f64), 0.0, 0.0);
    }
    component("priority", "admin priority", Some(priority as f64), priority_score(priority), weight)
}

pub(super) fn warming_weights(profile: &RoutingProfile) -> (f64, f64, f64, f64) {
    if priority_enabled(profile) {
        return (0.35, 0.25, 0.20, 0.20);
    }
    let scale = 1.0 / 0.65;
    (0.0, 0.25 * scale, 0.20 * scale, 0.20 * scale)
}

pub(super) fn component(code: &str, label: &str, raw_value: Option<f64>, normalized_score: f64, weight: f64) -> ScoreComponent {
    ScoreComponent {
        code: code.into(),
        label: label.into(),
        raw_value,
        normalized_score,
        weight,
        contribution: normalized_score * weight,
    }
}

pub(super) fn additive_component(code: &str, label: &str, contribution: f64) -> ScoreComponent {
    ScoreComponent {
        code: code.into(),
        label: label.into(),
        raw_value: None,
        normalized_score: contribution,
        weight: 1.0,
        contribution,
    }
}

fn normal_exploration_component(
    profile: &RoutingProfile,
    window: RoutingMetricWindow,
    total_attempts: u64,
    candidate: &RoutingScoreCandidate,
    recent_penalty: f64,
) -> Option<ScoreComponent> {
    if !normal_exploration_eligible(profile, window, candidate, recent_penalty) {
        return None;
    }
    let counts = super::scoring::exploration_counts(profile, total_attempts, candidate);
    let score = exploration_score(profile, counts.total_sample_count, counts.route_sample_count);
    if score <= f64::EPSILON {
        return None;
    }
    let contribution = (score * profile.exploration_weight).min(profile.exploration_cap);
    Some(capped_component(
        "exploration",
        "uncertainty exploration",
        Some(score),
        score,
        profile.exploration_weight,
        contribution,
    ))
}

fn ema_recent_component(profile: &RoutingProfile, candidate: &RoutingScoreCandidate) -> Option<ScoreComponent> {
    let ema = candidate.ema?;
    if ema.sample_count < profile.min_samples || ema.freshness_seconds > profile.ema_max_freshness_seconds {
        return None;
    }
    let ema_score = ema_composite_score(&profile.weights, ema)?;
    let window_score = window_composite_score(&profile.weights, &candidate.metric)?;
    let delta = ema_score - window_score;
    let contribution = (delta * profile.ema_recent_weight).clamp(-profile.ema_recent_cap, profile.ema_recent_cap);
    if contribution.abs() <= f64::EPSILON {
        return None;
    }
    Some(capped_component(
        "ema_recent",
        "EMA recent signal",
        Some(ema_score),
        delta,
        profile.ema_recent_weight,
        contribution,
    ))
}

fn normal_exploration_eligible(profile: &RoutingProfile, window: RoutingMetricWindow, candidate: &RoutingScoreCandidate, recent_penalty: f64) -> bool {
    recent_penalty <= 0.0 && !stale(candidate, window) && success_score(&candidate.metric) >= profile.exploration_min_success_score
}

fn ema_composite_score(weights: &RoutingProfileWeights, ema: RoutingEmaSnapshot) -> Option<f64> {
    let mut score = WeightedScore::default();
    score.push(clamp_score(ema.success_rate * 100.0), weights.success);
    score.push_optional(
        ema.ttfb_avg_ms.map(|value| lower_is_better(Some(value), TTFB_GOOD_MS, TTFB_BAD_MS)),
        weights.ttfb,
    );
    score.push_optional(
        ema.latency_avg_ms.map(|value| lower_is_better(Some(value), LATENCY_GOOD_MS, LATENCY_BAD_MS)),
        weights.latency,
    );
    score.push_optional(ema.output_tps.map(|value| higher_is_better(Some(value), TPS_LOW, TPS_HIGH)), weights.tps);
    score.finish()
}

fn window_composite_score(weights: &RoutingProfileWeights, metric: &RoutingMetricSnapshot) -> Option<f64> {
    let mut score = WeightedScore::default();
    score.push(success_score(metric), weights.success);
    score.push_optional(
        metric.ttfb_avg_ms.map(|value| lower_is_better(Some(value), TTFB_GOOD_MS, TTFB_BAD_MS)),
        weights.ttfb,
    );
    score.push_optional(
        metric.latency_avg_ms.map(|value| lower_is_better(Some(value), LATENCY_GOOD_MS, LATENCY_BAD_MS)),
        weights.latency,
    );
    score.push_optional(metric.output_tps.map(|value| higher_is_better(Some(value), TPS_LOW, TPS_HIGH)), weights.tps);
    score.finish()
}

fn capped_component(code: &str, label: &str, raw_value: Option<f64>, normalized_score: f64, weight: f64, contribution: f64) -> ScoreComponent {
    ScoreComponent {
        code: code.into(),
        label: label.into(),
        raw_value,
        normalized_score,
        weight,
        contribution,
    }
}

fn effective_priority_weight(profile: &RoutingProfile) -> f64 {
    if priority_enabled(profile) { profile.weights.priority } else { 0.0 }
}

fn priority_enabled(profile: &RoutingProfile) -> bool {
    profile.id == RoutingProfileId::FixedPriorityPlus
}

fn gap_ratio(current: Option<f64>, baseline: Option<f64>, ceiling: f64) -> f64 {
    let Some(current) = current else {
        return 0.0;
    };
    let Some(baseline) = baseline else {
        return 0.0;
    };
    ((current - baseline).max(0.0) / ceiling).clamp(0.0, 1.0)
}

#[derive(Default)]
struct WeightedScore {
    total: f64,
    weight: f64,
}

impl WeightedScore {
    fn push(&mut self, score: f64, weight: f64) {
        if weight <= f64::EPSILON {
            return;
        }
        self.total += score * weight;
        self.weight += weight;
    }

    fn push_optional(&mut self, score: Option<f64>, weight: f64) {
        if let Some(score) = score {
            self.push(score, weight);
        }
    }

    fn finish(self) -> Option<f64> {
        (self.weight > f64::EPSILON).then(|| self.total / self.weight)
    }
}
