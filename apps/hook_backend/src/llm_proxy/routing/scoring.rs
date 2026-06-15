mod math;

use rust_decimal::Decimal;
use types::provider::{
    RouteIdentity, RouteScoreExplanation, RoutingMetricSnapshot, RoutingMetricWindow, RoutingProfile, RoutingProfileId, RoutingRouteState, ScoreComponent,
};

use super::circuit::CircuitCandidateState;

use self::math::{
    CostRange, clamp_score, decimal_f64, exploration_score, final_score, headroom_ratio, headroom_score, higher_is_better, lower_is_better, priority_score,
    provider_health_prior, rate_limit_reason, score_order, soft_degraded, stale, success_rate, success_score,
};

const LATENCY_GOOD_MS: f64 = 800.0;
const LATENCY_BAD_MS: f64 = 12_000.0;
const TTFB_GOOD_MS: f64 = 250.0;
const TTFB_BAD_MS: f64 = 4_000.0;
const TPS_LOW: f64 = 5.0;
const TPS_HIGH: f64 = 120.0;

#[derive(Clone, Debug)]
pub(crate) struct RoutingScoreCandidate {
    pub(crate) route: RouteIdentity,
    pub(crate) provider_name: Option<String>,
    pub(crate) key_name: Option<String>,
    pub(crate) key_preview: Option<String>,
    pub(crate) endpoint_name: Option<String>,
    pub(crate) metric: RoutingMetricSnapshot,
    pub(crate) metric_window: RoutingMetricWindow,
    pub(crate) metric_freshness_seconds: i64,
    pub(crate) recent_metric: Option<RoutingMetricSnapshot>,
    pub(crate) circuit_state: CircuitCandidateState,
    pub(crate) admin_priority: i32,
    pub(crate) estimated_cost: Option<Decimal>,
    pub(crate) needs_conversion: bool,
    pub(crate) is_cached: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ScoredRoute {
    pub(crate) original_index: usize,
    pub(crate) explanation: RouteScoreExplanation,
    pub(crate) excluded: bool,
}

pub(crate) fn score_routes(profile: &RoutingProfile, window: RoutingMetricWindow, candidates: Vec<RoutingScoreCandidate>) -> Vec<ScoredRoute> {
    let total_attempts = candidates.iter().map(|item| item.metric.sample_count).sum::<u64>();
    let cost_range = CostRange::from_candidates(&candidates);
    let mut scored = candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| score_route(index, profile, window, total_attempts, cost_range, candidate))
        .collect::<Vec<_>>();
    scored.sort_by(score_order);
    for (index, item) in scored.iter_mut().enumerate() {
        item.explanation.rank = (index + 1).try_into().unwrap_or(u32::MAX);
    }
    scored
}

fn score_route(
    index: usize,
    profile: &RoutingProfile,
    window: RoutingMetricWindow,
    total_attempts: u64,
    cost_range: CostRange,
    candidate: RoutingScoreCandidate,
) -> ScoredRoute {
    if let Some(excluded) = hard_exclusion(index, candidate.clone()) {
        return excluded;
    }
    let context = ScoreContext {
        profile,
        window,
        total_attempts,
        cost_range,
    };
    if candidate.metric.sample_count < profile.min_samples {
        return warming_score(index, context, candidate);
    }
    normal_score(index, context, candidate)
}

fn hard_exclusion(index: usize, candidate: RoutingScoreCandidate) -> Option<ScoredRoute> {
    let reason = match &candidate.circuit_state {
        CircuitCandidateState::Open { reason, ttl_seconds } => Some(format!("{reason}; ttl={ttl_seconds}s")),
        CircuitCandidateState::HalfOpenBusy { reason } => Some(reason.clone()),
        CircuitCandidateState::Closed | CircuitCandidateState::HalfOpenProbe { .. } => rate_limit_reason(&candidate.metric),
    }?;
    let state = if matches!(candidate.circuit_state, CircuitCandidateState::Open { .. }) {
        RoutingRouteState::CircuitOpen
    } else {
        RoutingRouteState::Excluded
    };
    Some(ScoredRoute {
        original_index: index,
        excluded: true,
        explanation: explanation(candidate, state, 0.0, reason.clone(), Vec::new(), Some(reason)),
    })
}

fn warming_score(index: usize, context: ScoreContext<'_>, candidate: RoutingScoreCandidate) -> ScoredRoute {
    let priority = priority_score(candidate.admin_priority);
    let cost = context.cost_range.score(candidate.estimated_cost);
    let health = provider_health_prior(&candidate.metric);
    let exploration = exploration_score(context.profile, context.total_attempts, candidate.metric.sample_count);
    let (priority_weight, cost_weight, health_weight, exploration_weight) = warming_weights(context.profile);
    let mut components = vec![
        component("priority", "admin priority", Some(candidate.admin_priority as f64), priority, priority_weight),
        component("cost", "configured cost", candidate.estimated_cost.and_then(decimal_f64), cost, cost_weight),
        component("health_prior", "health prior", None, health, health_weight),
        component("exploration", "uncertainty exploration", Some(exploration), exploration, exploration_weight),
    ];
    if candidate.is_cached {
        components.push(additive_component("affinity", "cache affinity", context.profile.affinity_bonus));
    }
    let score = clamp_score(components.iter().map(|item| item.contribution).sum());
    ScoredRoute {
        original_index: index,
        excluded: false,
        explanation: explanation(
            candidate,
            RoutingRouteState::Warming,
            score,
            selected_reason(score, &components),
            components,
            None,
        ),
    }
}

fn normal_score(index: usize, context: ScoreContext<'_>, candidate: RoutingScoreCandidate) -> ScoredRoute {
    let components = normal_components(context, &candidate);
    let penalty = penalty_components(context.profile, context.window, &candidate);
    let score = final_score(&components, &penalty);
    let mut all_components = components;
    all_components.extend(penalty);
    let state = soft_state(context.profile, &candidate, context.window);
    ScoredRoute {
        original_index: index,
        excluded: false,
        explanation: explanation(candidate, state, score, selected_reason(score, &all_components), all_components, None),
    }
}

fn normal_components(context: ScoreContext<'_>, candidate: &RoutingScoreCandidate) -> Vec<ScoreComponent> {
    let weights = &context.profile.weights;
    vec![
        component(
            "success",
            "success rate",
            Some(success_rate(&candidate.metric)),
            success_score(&candidate.metric),
            weights.success,
        ),
        component(
            "ttfb",
            "avg TTFB",
            candidate.metric.ttfb_avg_ms,
            lower_is_better(candidate.metric.ttfb_avg_ms, TTFB_GOOD_MS, TTFB_BAD_MS),
            weights.ttfb,
        ),
        component(
            "latency",
            "avg latency",
            candidate.metric.latency_avg_ms,
            lower_is_better(candidate.metric.latency_avg_ms, LATENCY_GOOD_MS, LATENCY_BAD_MS),
            weights.latency,
        ),
        component(
            "tps",
            "output TPS",
            candidate.metric.output_tps,
            higher_is_better(candidate.metric.output_tps, TPS_LOW, TPS_HIGH),
            weights.tps,
        ),
        component(
            "cost",
            "configured cost",
            candidate.estimated_cost.and_then(decimal_f64),
            context.cost_range.score(candidate.estimated_cost),
            weights.cost,
        ),
        component(
            "headroom",
            "RPM headroom",
            Some(headroom_ratio(&candidate.metric)),
            headroom_score(&candidate.metric),
            weights.headroom,
        ),
        priority_component(context.profile, candidate.admin_priority),
    ]
}

fn penalty_components(profile: &RoutingProfile, window: RoutingMetricWindow, candidate: &RoutingScoreCandidate) -> Vec<ScoreComponent> {
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
    if let CircuitCandidateState::HalfOpenProbe { .. } = candidate.circuit_state {
        output.push(additive_component("half_open", "half-open probe", -profile.stale_metric_penalty));
    }
    let recent_penalty = recent_regression_penalty(profile, candidate, window);
    if recent_penalty > 0.0 {
        output.push(additive_component("recent_regression", "recent window regression", -recent_penalty));
    }
    output
}

#[derive(Clone, Copy)]
struct ScoreContext<'a> {
    profile: &'a RoutingProfile,
    window: RoutingMetricWindow,
    total_attempts: u64,
    cost_range: CostRange,
}

fn explanation(
    candidate: RoutingScoreCandidate,
    state: RoutingRouteState,
    final_score: f64,
    selected_reason: String,
    components: Vec<ScoreComponent>,
    exclusion_reason: Option<String>,
) -> RouteScoreExplanation {
    RouteScoreExplanation {
        route: candidate.route,
        provider_name: candidate.provider_name,
        key_name: candidate.key_name,
        key_preview: candidate.key_preview,
        endpoint_name: candidate.endpoint_name,
        rank: 0,
        state,
        final_score,
        metric_window: candidate.metric_window,
        selected_reason,
        components,
        raw_metrics: candidate.metric,
        exclusion_reason,
        metric_freshness_seconds: candidate.metric_freshness_seconds,
    }
}

fn selected_reason(score: f64, components: &[ScoreComponent]) -> String {
    let parts = components
        .iter()
        .filter(|component| component.contribution.abs() > 0.01)
        .map(|component| format!("{} {:+.1}", component.label, component.contribution))
        .collect::<Vec<_>>()
        .join(", ");
    format!("score {score:.1}: {parts}")
}

fn priority_component(profile: &RoutingProfile, priority: i32) -> ScoreComponent {
    let weight = effective_priority_weight(profile);
    if weight <= f64::EPSILON {
        return component("priority", "admin priority", Some(priority as f64), 0.0, 0.0);
    }
    component("priority", "admin priority", Some(priority as f64), priority_score(priority), weight)
}

fn warming_weights(profile: &RoutingProfile) -> (f64, f64, f64, f64) {
    if priority_enabled(profile) {
        return (0.35, 0.25, 0.20, 0.20);
    }
    let scale = 1.0 / 0.65;
    (0.0, 0.25 * scale, 0.20 * scale, 0.20 * scale)
}

fn effective_priority_weight(profile: &RoutingProfile) -> f64 {
    if priority_enabled(profile) { profile.weights.priority } else { 0.0 }
}

fn priority_enabled(profile: &RoutingProfile) -> bool {
    profile.id == RoutingProfileId::FixedPriorityPlus
}

fn component(code: &str, label: &str, raw_value: Option<f64>, normalized_score: f64, weight: f64) -> ScoreComponent {
    ScoreComponent {
        code: code.into(),
        label: label.into(),
        raw_value,
        normalized_score,
        weight,
        contribution: normalized_score * weight,
    }
}

fn additive_component(code: &str, label: &str, contribution: f64) -> ScoreComponent {
    ScoreComponent {
        code: code.into(),
        label: label.into(),
        raw_value: None,
        normalized_score: contribution,
        weight: 1.0,
        contribution,
    }
}

fn soft_state(profile: &RoutingProfile, candidate: &RoutingScoreCandidate, window: RoutingMetricWindow) -> RoutingRouteState {
    if recent_regression_penalty(profile, candidate, window) > 0.0 {
        return RoutingRouteState::Degraded;
    }
    if soft_degraded(candidate, window) {
        return RoutingRouteState::Degraded;
    }
    RoutingRouteState::Eligible
}

fn recent_regression_penalty(profile: &RoutingProfile, candidate: &RoutingScoreCandidate, window: RoutingMetricWindow) -> f64 {
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
    let penalty = (success_gap * 0.6 + latency_gap * 0.2 + ttfb_gap * 0.2) * profile.stale_metric_penalty;
    clamp_score(penalty)
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
