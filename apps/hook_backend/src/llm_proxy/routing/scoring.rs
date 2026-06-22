pub(super) mod math;

use rust_decimal::Decimal;
use types::provider::{
    RouteIdentity, RouteScoreExplanation, RoutingMetricSnapshot, RoutingMetricSource, RoutingMetricWindow, RoutingPriorSource, RoutingProfile,
    RoutingRequestFeatures, RoutingRouteState, ScoreComponent,
};

use super::circuit::CircuitCandidateState;

use self::math::{
    CostRange, clamp_score, decimal_f64, exploration_score, final_score, headroom_ratio, headroom_score, higher_is_better, lower_is_better, priority_score,
    provider_health_prior, rate_limit_reason, score_order, success_rate, success_score,
};
use super::scoring_support::{
    additive_component, component, normal_adjustment_components, penalty_components, priority_component, recent_regression_penalty, selected_reason,
    soft_state, warming_weights,
};

pub(super) const LATENCY_GOOD_MS: f64 = 800.0;
pub(super) const LATENCY_BAD_MS: f64 = 12_000.0;
pub(super) const TTFB_GOOD_MS: f64 = 250.0;
pub(super) const TTFB_BAD_MS: f64 = 4_000.0;
pub(super) const TPS_LOW: f64 = 5.0;
pub(super) const TPS_HIGH: f64 = 120.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RoutingEmaSnapshot {
    pub(crate) success_rate: f64,
    pub(crate) ttfb_avg_ms: Option<f64>,
    pub(crate) latency_avg_ms: Option<f64>,
    pub(crate) output_tps: Option<f64>,
    pub(crate) sample_count: u64,
    pub(crate) freshness_seconds: i64,
}

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
    pub(crate) metric_source: RoutingMetricSource,
    pub(crate) prior_source: RoutingPriorSource,
    pub(crate) prior_sample_count: u64,
    pub(crate) effective_sample_count: u64,
    pub(crate) routing_context_key: Option<String>,
    pub(crate) route_config_fingerprint: Option<String>,
    pub(crate) price_config_fingerprint: Option<String>,
    pub(crate) context_route_sample_count: u64,
    pub(crate) context_total_sample_count: u64,
    pub(crate) ema: Option<RoutingEmaSnapshot>,
    pub(crate) circuit_state: CircuitCandidateState,
    pub(crate) admin_priority: i32,
    pub(crate) estimated_cost: Option<Decimal>,
    pub(crate) needs_conversion: bool,
    pub(crate) is_cached: bool,
    pub(crate) request_features: RoutingRequestFeatures,
}

#[derive(Clone, Debug)]
pub(crate) struct ScoredRoute {
    pub(crate) original_index: usize,
    pub(crate) explanation: RouteScoreExplanation,
    pub(crate) excluded: bool,
}

pub(crate) fn score_routes(profile: &RoutingProfile, window: RoutingMetricWindow, candidates: Vec<RoutingScoreCandidate>) -> Vec<ScoredRoute> {
    let total_attempts = candidates.iter().map(|item| item.effective_sample_count).sum::<u64>();
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
    if candidate.effective_sample_count < profile.min_samples {
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
    let exploration_counts = exploration_counts(context.profile, context.total_attempts, &candidate);
    let exploration = exploration_score(context.profile, exploration_counts.total_sample_count, exploration_counts.route_sample_count);
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
    let recent_penalty = recent_regression_penalty(context.profile, &candidate, context.window);
    let mut components = normal_components(context, &candidate);
    components.extend(normal_adjustment_components(
        context.profile,
        context.window,
        context.total_attempts,
        &candidate,
        recent_penalty,
    ));
    let penalty = penalty_components(context.profile, context.window, &candidate, recent_penalty);
    let score = final_score(&components, &penalty);
    let mut all_components = components;
    all_components.extend(penalty);
    let state = soft_state(&candidate, context.window, recent_penalty);
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
        metric_source: candidate.metric_source,
        prior_source: candidate.prior_source,
        prior_sample_count: candidate.prior_sample_count,
        effective_sample_count: candidate.effective_sample_count,
        routing_context_key: candidate.routing_context_key,
        route_config_fingerprint: candidate.route_config_fingerprint,
        price_config_fingerprint: candidate.price_config_fingerprint,
        request_features: candidate.request_features,
    }
}

pub(in crate::llm_proxy::routing) fn exploration_counts(profile: &RoutingProfile, total_attempts: u64, candidate: &RoutingScoreCandidate) -> ExplorationCounts {
    if profile.contextual_exploration_enabled {
        return ExplorationCounts {
            total_sample_count: candidate.context_total_sample_count,
            route_sample_count: candidate.context_route_sample_count,
        };
    }
    ExplorationCounts {
        total_sample_count: total_attempts,
        route_sample_count: candidate.effective_sample_count,
    }
}

pub(in crate::llm_proxy::routing) struct ExplorationCounts {
    pub(in crate::llm_proxy::routing) total_sample_count: u64,
    pub(in crate::llm_proxy::routing) route_sample_count: u64,
}
