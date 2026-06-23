use rust_decimal::{Decimal, prelude::ToPrimitive};
use types::provider::{RoutingMetricSnapshot, RoutingMetricWindow, RoutingProfile, ScoreComponent};

use super::{RoutingScoreCandidate, ScoredRoute};
use crate::llm_proxy::routing::circuit::CircuitCandidateState;

const PRIOR_SUCCESS: f64 = 1.0;
const PRIOR_FAIL: f64 = 1.0;
const PRIORITY_MAX: f64 = 1_000.0;
const STALE_MULTIPLIER: i64 = 2;

#[derive(Clone, Copy)]
pub(super) struct CostRange {
    min: Option<Decimal>,
    max: Option<Decimal>,
}

impl CostRange {
    pub(super) fn from_candidates(candidates: &[RoutingScoreCandidate]) -> Self {
        let mut values = candidates.iter().filter_map(|item| item.estimated_cost);
        let Some(first) = values.next() else {
            return Self { min: None, max: None };
        };
        let (min, max) = values.fold((first, first), |(min, max), value| (min.min(value), max.max(value)));
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub(super) fn score(self, value: Option<Decimal>) -> f64 {
        let Some(value) = value.and_then(decimal_f64) else {
            return 50.0;
        };
        let Some(min) = self.min.and_then(decimal_f64) else {
            return 100.0;
        };
        let Some(max) = self.max.and_then(decimal_f64) else {
            return 100.0;
        };
        if (max - min).abs() < f64::EPSILON {
            return 100.0;
        }
        clamp_score((max - value) * 100.0 / (max - min))
    }
}

pub(super) fn score_order(left: &ScoredRoute, right: &ScoredRoute) -> std::cmp::Ordering {
    (left.excluded, ReverseScore(left.explanation.final_score), left.original_index).cmp(&(
        right.excluded,
        ReverseScore(right.explanation.final_score),
        right.original_index,
    ))
}

#[derive(Clone, Copy, PartialEq)]
struct ReverseScore(f64);

impl Eq for ReverseScore {}

impl Ord for ReverseScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.partial_cmp(&self.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for ReverseScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub(super) fn final_score(components: &[ScoreComponent], penalty: &[ScoreComponent]) -> f64 {
    clamp_score(components.iter().chain(penalty).map(|item| item.contribution).sum())
}

pub(in crate::llm_proxy::routing) fn soft_degraded(candidate: &RoutingScoreCandidate, window: RoutingMetricWindow) -> bool {
    matches!(candidate.circuit_state, CircuitCandidateState::HalfOpenProbe { .. }) || stale(candidate, window) || success_score(&candidate.metric) < 75.0
}

pub(in crate::llm_proxy::routing) fn stale(candidate: &RoutingScoreCandidate, window: RoutingMetricWindow) -> bool {
    candidate.metric.sample_count > 0 && candidate.metric_freshness_seconds > window.seconds() * STALE_MULTIPLIER
}

pub(super) fn rate_limit_reason(metric: &RoutingMetricSnapshot) -> Option<String> {
    let limit = metric.rpm_limit?;
    (metric.rpm_used >= limit).then(|| "provider_key_rate_limit_exhausted".into())
}

pub(super) fn provider_health_prior(metric: &RoutingMetricSnapshot) -> f64 {
    if metric.sample_count == 0 {
        return 80.0;
    }
    success_score(metric)
}

pub(in crate::llm_proxy::routing) fn exploration_score(profile: &RoutingProfile, total_attempts: u64, route_attempts: u64) -> f64 {
    let bonus = profile.exploration_k * (((total_attempts + 1) as f64).ln() / (route_attempts + 1) as f64).sqrt();
    clamp_score(bonus * 20.0)
}

pub(in crate::llm_proxy::routing) fn success_score(metric: &RoutingMetricSnapshot) -> f64 {
    let (success_count, request_count) = effective_success_counts(metric);
    let success = success_count as f64 + PRIOR_SUCCESS;
    let attempts = request_count as f64 + PRIOR_SUCCESS + PRIOR_FAIL;
    clamp_score(100.0 * success / attempts)
}

pub(super) fn success_rate(metric: &RoutingMetricSnapshot) -> f64 {
    let (success_count, request_count) = effective_success_counts(metric);
    if request_count == 0 {
        return 0.0;
    }
    success_count as f64 / request_count as f64
}

pub(in crate::llm_proxy::routing) fn lower_is_better(value: Option<f64>, good: f64, bad: f64) -> f64 {
    value.map(|value| clamp_score((bad - value) * 100.0 / (bad - good))).unwrap_or(50.0)
}

pub(in crate::llm_proxy::routing) fn higher_is_better(value: Option<f64>, low: f64, high: f64) -> f64 {
    value.map(|value| clamp_score((value - low) * 100.0 / (high - low))).unwrap_or(50.0)
}

pub(super) fn headroom_score(metric: &RoutingMetricSnapshot) -> f64 {
    metric.rpm_limit.map(|_| clamp_score(headroom_ratio(metric) * 100.0)).unwrap_or(100.0)
}

pub(super) fn headroom_ratio(metric: &RoutingMetricSnapshot) -> f64 {
    let Some(limit) = metric.rpm_limit else {
        return 1.0;
    };
    if limit == 0 {
        return 0.0;
    }
    limit.saturating_sub(metric.rpm_used) as f64 / limit as f64
}

pub(in crate::llm_proxy::routing) fn priority_score(priority: i32) -> f64 {
    let value = f64::from(priority.max(0));
    clamp_score((PRIORITY_MAX - value) * 100.0 / PRIORITY_MAX)
}

pub(in crate::llm_proxy::routing) fn clamp_score(value: f64) -> f64 {
    value.clamp(0.0, 100.0)
}

pub(super) fn decimal_f64(value: Decimal) -> Option<f64> {
    value.to_f64()
}

fn effective_success_counts(metric: &RoutingMetricSnapshot) -> (u64, u64) {
    let first_output_attempts = metric.first_output_success_count.saturating_add(metric.first_output_failure_count);
    if first_output_attempts > 0 {
        return (metric.first_output_success_count, first_output_attempts);
    }
    (metric.success_count, metric.request_count)
}
