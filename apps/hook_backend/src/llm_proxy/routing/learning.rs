use storage::provider::{ProviderStore, RoutingMetricRecord, RoutingProfileVersionSnapshot};
use time::{Duration, OffsetDateTime};
use types::provider::{RoutingMetricWindow, RoutingProfile, RoutingProfileLearningState, RoutingProfileWeights};

use super::learning_cost::{CostRange, CurrentCostCatalog, current_costs};
use crate::llm_proxy::{LlmProxyError, LlmProxyState};

const AUTO_TUNE_STRENGTH: f64 = 0.35;
const LEARNING_RATE: f64 = 0.08;
const LEARNING_REFRESH: Duration = Duration::minutes(15);
const LEARNING_WINDOW: RoutingMetricWindow = RoutingMetricWindow::SevenDays;
const MIN_ROUTE_SAMPLES: u64 = 10;
const WARMUP_SAMPLE_COUNT: u64 = 800;
const LATENCY_GOOD_MS: f64 = 800.0;
const LATENCY_BAD_MS: f64 = 12_000.0;
const TTFB_GOOD_MS: f64 = 250.0;
const TTFB_BAD_MS: f64 = 4_000.0;
const TPS_LOW: f64 = 5.0;
const TPS_HIGH: f64 = 120.0;
const QUALITY_PENALTY_WEIGHT: f64 = 0.20;

pub(crate) async fn apply_profile_learning(state: &LlmProxyState, profile: RoutingProfile) -> Result<RoutingProfile, LlmProxyError> {
    if !profile.auto_tune_enabled {
        return Ok(profile);
    }
    let store = ProviderStore::new(state.database.clone());
    let now = OffsetDateTime::now_utc();
    let latest = store.get_latest_routing_profile_version(profile.id.as_str()).await?;
    let snapshot = if needs_refresh(latest.as_ref(), &profile, now) {
        let refreshed = build_snapshot(state, &store, &profile, now).await?;
        store.insert_routing_profile_version_snapshot(&refreshed).await?;
        refreshed
    } else {
        latest.expect("latest routing profile version must exist when refresh is skipped")
    };
    Ok(hydrate_profile(profile, snapshot))
}

fn needs_refresh(latest: Option<&RoutingProfileVersionSnapshot>, profile: &RoutingProfile, now: OffsetDateTime) -> bool {
    let Some(latest) = latest else {
        return true;
    };
    latest.sample_count == 0 || latest.admin_weights != profile.weights || now - latest.created_at >= LEARNING_REFRESH
}

async fn build_snapshot(
    state: &LlmProxyState,
    store: &ProviderStore,
    profile: &RoutingProfile,
    now: OffsetDateTime,
) -> Result<RoutingProfileVersionSnapshot, LlmProxyError> {
    let metrics = store.list_routing_metrics(LEARNING_WINDOW).await?;
    let sample_count = metrics.iter().map(|item| item.snapshot.sample_count).sum::<u64>();
    let current_costs = current_costs(state, &metrics).await?;
    let learned_weights = learn_weights(profile, &metrics, &current_costs);
    let confidence = confidence(sample_count);
    let effective_weights = learned_weights
        .as_ref()
        .map(|learned| blend_weights(&profile.weights, learned, AUTO_TUNE_STRENGTH * confidence))
        .unwrap_or_else(|| profile.weights.clone());
    Ok(RoutingProfileVersionSnapshot {
        profile_id: profile.id.as_str().to_owned(),
        profile_version: learned_version(&profile.version, now),
        admin_weights: profile.weights.clone(),
        learned_weights,
        effective_weights,
        reward_window: LEARNING_WINDOW,
        sample_count,
        created_at: now,
    })
}

fn hydrate_profile(mut profile: RoutingProfile, snapshot: RoutingProfileVersionSnapshot) -> RoutingProfile {
    let confidence = confidence(snapshot.sample_count);
    profile.version = snapshot.profile_version.clone();
    profile.weights = snapshot.effective_weights.clone();
    profile.learning = Some(RoutingProfileLearningState {
        admin_weights: snapshot.admin_weights,
        learned_weights: snapshot.learned_weights,
        effective_weights: snapshot.effective_weights,
        reward_window: snapshot.reward_window,
        sample_count: snapshot.sample_count,
        confidence,
        updated_at: format_timestamp(snapshot.created_at),
    });
    profile
}

fn learn_weights(profile: &RoutingProfile, metrics: &[RoutingMetricRecord], current_costs: &CurrentCostCatalog) -> Option<RoutingProfileWeights> {
    let eligible = metrics
        .iter()
        .filter(|item| item.snapshot.sample_count >= MIN_ROUTE_SAMPLES)
        .collect::<Vec<_>>();
    if eligible.is_empty() {
        return None;
    }
    let cost_range = CostRange::from_records(&eligible, current_costs);
    let mut learned = WeightVector::from_weights(&profile.weights);
    for record in eligible {
        let signals = SignalVector::from_record(record, cost_range, current_costs.get(&record.route).copied());
        let reward = reward(profile, signals);
        learned = learned.update(signals, reward);
    }
    Some(learned.normalize_with_locked(WeightVector::from_weights(&profile.weights)).into_weights())
}

fn reward(profile: &RoutingProfile, signals: SignalVector) -> f64 {
    let success_gain = signals.success;
    let failure_penalty = 1.0 - signals.success;
    let ttfb_penalty = (1.0 - signals.ttfb) * profile.weights.ttfb;
    let latency_penalty = (1.0 - signals.latency) * profile.weights.latency;
    let cost_penalty = (1.0 - signals.cost) * profile.weights.cost;
    let quality_penalty = (1.0 - signals.quality) * QUALITY_PENALTY_WEIGHT;
    let tps_reward = signals.tps * profile.weights.tps;
    (success_gain - failure_penalty - ttfb_penalty - latency_penalty - cost_penalty - quality_penalty + tps_reward).clamp(-1.0, 1.0)
}

pub(super) fn blend_weights(admin: &RoutingProfileWeights, learned: &RoutingProfileWeights, strength: f64) -> RoutingProfileWeights {
    WeightVector::from_weights(admin)
        .blend_learnable(WeightVector::from_weights(learned), strength.clamp(0.0, 1.0))
        .normalize_with_locked(WeightVector::from_weights(admin))
        .into_weights()
}

fn confidence(sample_count: u64) -> f64 {
    (sample_count as f64 / WARMUP_SAMPLE_COUNT as f64).clamp(0.0, 1.0)
}

fn learned_version(base: &str, now: OffsetDateTime) -> String {
    let compact = if base.len() > 32 { &base[..32] } else { base }.trim_end_matches('-');
    format!("{compact}-auto-{}", now.unix_timestamp())
}

fn format_timestamp(value: OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .expect("routing profile learning timestamp must format as RFC3339")
}

#[derive(Clone, Copy)]
struct SignalVector {
    success: f64,
    ttfb: f64,
    latency: f64,
    tps: f64,
    cost: f64,
    quality: f64,
}

impl SignalVector {
    fn from_record(record: &RoutingMetricRecord, cost_range: CostRange, current_cost: Option<f64>) -> Self {
        Self {
            success: success_score(record),
            ttfb: lower_is_better(record.snapshot.ttfb_avg_ms, TTFB_GOOD_MS, TTFB_BAD_MS),
            latency: lower_is_better(record.snapshot.latency_avg_ms, LATENCY_GOOD_MS, LATENCY_BAD_MS),
            tps: higher_is_better(record.snapshot.output_tps, TPS_LOW, TPS_HIGH),
            cost: cost_range.score(current_cost),
            quality: quality_score(record),
        }
    }
}

#[derive(Clone, Copy)]
struct WeightVector {
    success: f64,
    ttfb: f64,
    latency: f64,
    tps: f64,
    cost: f64,
    headroom: f64,
    priority: f64,
}

impl WeightVector {
    fn from_weights(weights: &RoutingProfileWeights) -> Self {
        Self {
            success: weights.success,
            ttfb: weights.ttfb,
            latency: weights.latency,
            tps: weights.tps,
            cost: weights.cost,
            headroom: weights.headroom,
            priority: weights.priority,
        }
    }

    fn into_weights(self) -> RoutingProfileWeights {
        RoutingProfileWeights {
            success: self.success,
            ttfb: self.ttfb,
            latency: self.latency,
            tps: self.tps,
            cost: self.cost,
            headroom: self.headroom,
            priority: self.priority,
        }
    }

    fn normalize_with_locked(self, baseline: Self) -> Self {
        let locked_headroom = baseline.headroom.max(0.0);
        let locked_priority = baseline.priority.max(0.0);
        let learnable_budget = (1.0 - locked_headroom - locked_priority).max(0.0);
        let candidate = if self.learnable_sum() > f64::EPSILON { self } else { baseline };
        let learnable_sum = candidate.learnable_sum();
        if learnable_sum <= f64::EPSILON {
            return baseline;
        }
        let scale = learnable_budget / learnable_sum;
        Self {
            success: candidate.success * scale,
            ttfb: candidate.ttfb * scale,
            latency: candidate.latency * scale,
            tps: candidate.tps * scale,
            cost: candidate.cost * scale,
            headroom: locked_headroom,
            priority: locked_priority,
        }
    }

    fn blend_learnable(self, other: Self, strength: f64) -> Self {
        let keep = 1.0 - strength;
        Self {
            success: self.success * keep + other.success * strength,
            ttfb: self.ttfb * keep + other.ttfb * strength,
            latency: self.latency * keep + other.latency * strength,
            tps: self.tps * keep + other.tps * strength,
            cost: self.cost * keep + other.cost * strength,
            headroom: self.headroom,
            priority: self.priority,
        }
    }

    fn update(self, signals: SignalVector, reward: f64) -> Self {
        Self {
            success: update_weight(self.success, signals.success, reward),
            ttfb: update_weight(self.ttfb, signals.ttfb, reward),
            latency: update_weight(self.latency, signals.latency, reward),
            tps: update_weight(self.tps, signals.tps, reward),
            cost: update_weight(self.cost, signals.cost, reward),
            headroom: self.headroom,
            priority: self.priority,
        }
    }

    fn learnable_sum(self) -> f64 {
        self.success + self.ttfb + self.latency + self.tps + self.cost
    }
}

fn update_weight(weight: f64, signal: f64, reward: f64) -> f64 {
    let centered = signal * 2.0 - 1.0;
    (weight.max(0.000_1) * (LEARNING_RATE * reward * centered).exp()).max(0.000_1)
}

fn success_score(record: &RoutingMetricRecord) -> f64 {
    let success = record.snapshot.success_count as f64 + 1.0;
    let attempts = record.snapshot.request_count as f64 + 2.0;
    (success / attempts).clamp(0.0, 1.0)
}

pub(super) fn quality_score(record: &RoutingMetricRecord) -> f64 {
    let attempts = record.snapshot.request_count.max(1) as f64;
    let failures = record.snapshot.format_conversion_failure_count
        + record.snapshot.usage_missing_count
        + record.snapshot.stream_abnormal_end_count
        + record.snapshot.schema_tool_call_failure_count;
    (1.0 - failures as f64 / attempts).clamp(0.0, 1.0)
}

fn lower_is_better(value: Option<f64>, good: f64, bad: f64) -> f64 {
    value.map(|item| ((bad - item) / (bad - good)).clamp(0.0, 1.0)).unwrap_or(0.5)
}

fn higher_is_better(value: Option<f64>, low: f64, high: f64) -> f64 {
    value.map(|item| ((item - low) / (high - low)).clamp(0.0, 1.0)).unwrap_or(0.5)
}
