use std::collections::HashMap;

use storage::provider::ProviderStore;
use types::provider::{
    DEFAULT_EMA_REGRESSION_PENALTY, DEFAULT_EXPLORATION_BUDGET_PERCENT, RoutingCacheAffinityMode, RoutingProfile, RoutingProfileId, RoutingProfileUpsert,
    RoutingProfileWeights,
};

use crate::llm_proxy::{LlmProxyError, LlmProxyState};

use super::learning::apply_profile_learning;

const BUILTIN_PROFILE_VERSION: &str = "builtin-v1";
const PROFILE_WEIGHT_SUM: f64 = 1.0;
const WEIGHT_SUM_TOLERANCE: f64 = 0.001;
const PERCENT_MIN: f64 = 0.0;
const PERCENT_MAX: f64 = 100.0;

#[derive(Clone, Debug)]
pub(crate) struct VersionedRoutingProfile {
    pub(crate) profile: RoutingProfile,
}

pub(crate) async fn list_profiles(state: &LlmProxyState) -> Result<Vec<RoutingProfile>, LlmProxyError> {
    let mut output = Vec::new();
    for profile in raw_profiles(state).await? {
        output.push(apply_profile_learning(state, profile).await?);
    }
    Ok(output)
}

pub(crate) async fn profile_by_id(state: &LlmProxyState, id: RoutingProfileId) -> Result<VersionedRoutingProfile, LlmProxyError> {
    let profile = raw_profiles(state)
        .await?
        .into_iter()
        .find(|profile| profile.id == id)
        .unwrap_or_else(|| built_in_profile(id));
    Ok(VersionedRoutingProfile {
        profile: apply_profile_learning(state, profile).await?,
    })
}

pub(crate) async fn upsert_profile(state: &LlmProxyState, id: RoutingProfileId, patch: RoutingProfileUpsert) -> Result<RoutingProfile, LlmProxyError> {
    let mut profile = raw_profiles(state)
        .await?
        .into_iter()
        .find(|profile| profile.id == id)
        .unwrap_or_else(|| built_in_profile(id));
    apply_patch(&mut profile, patch);
    enforce_priority_policy(&mut profile);
    validate_profile(&profile)?;
    profile.learning = None;
    profile.version = new_version();
    ProviderStore::new(state.database.clone())
        .upsert_routing_profile_overlay(profile)
        .await
        .map_err(LlmProxyError::from)
}

pub(crate) fn profile_id_from_str(value: &str) -> RoutingProfileId {
    RoutingProfileId::from(value)
}

async fn overlays(state: &LlmProxyState) -> Result<Vec<RoutingProfile>, LlmProxyError> {
    ProviderStore::new(state.database.clone())
        .list_routing_profile_overlays()
        .await
        .map_err(LlmProxyError::from)
}

async fn raw_profiles(state: &LlmProxyState) -> Result<Vec<RoutingProfile>, LlmProxyError> {
    let mut profiles = built_in_profiles();
    for overlay in overlays(state).await? {
        upsert_profile_item(&mut profiles, overlay);
    }
    profiles.iter_mut().for_each(enforce_priority_policy);
    Ok(profiles)
}

fn apply_patch(profile: &mut RoutingProfile, patch: RoutingProfileUpsert) {
    if let Some(weights) = patch.weights {
        profile.weights = weights;
    }
    if let Some(value) = patch.min_samples {
        profile.min_samples = value;
    }
    if let Some(value) = patch.exploration_k {
        profile.exploration_k = value;
    }
    if let Some(value) = patch.exploration_budget_percent {
        profile.exploration_budget_percent = value;
    }
    if let Some(value) = patch.conversion_penalty {
        profile.conversion_penalty = value;
    }
    if let Some(value) = patch.stale_metric_penalty {
        profile.stale_metric_penalty = value;
    }
    if let Some(value) = patch.ema_regression_penalty {
        profile.ema_regression_penalty = value;
    }
    if let Some(value) = patch.cache_affinity_mode {
        profile.cache_affinity_mode = value;
    }
    if let Some(value) = patch.affinity_bonus {
        profile.affinity_bonus = value;
    }
    if let Some(value) = patch.auto_tune_enabled {
        profile.auto_tune_enabled = value;
    }
}

fn built_in_profiles() -> Vec<RoutingProfile> {
    [
        built_in_profile(RoutingProfileId::Balanced),
        built_in_profile(RoutingProfileId::FirstByte),
        built_in_profile(RoutingProfileId::HighTps),
        built_in_profile(RoutingProfileId::CostOptimal),
        built_in_profile(RoutingProfileId::HighAvailability),
        built_in_profile(RoutingProfileId::CacheAffinityPlus),
        built_in_profile(RoutingProfileId::FixedPriorityPlus),
        built_in_profile(RoutingProfileId::Custom),
    ]
    .into()
}

fn built_in_profile(id: RoutingProfileId) -> RoutingProfile {
    let (name, description, weights) = built_in_definition(id);
    let (cache_affinity_mode, affinity_bonus) = built_in_affinity(id);
    let mut profile = RoutingProfile {
        id,
        name: name.into(),
        description: description.into(),
        weights,
        version: BUILTIN_PROFILE_VERSION.into(),
        min_samples: 20,
        exploration_k: 3.0,
        exploration_budget_percent: DEFAULT_EXPLORATION_BUDGET_PERCENT,
        conversion_penalty: 6.0,
        stale_metric_penalty: 8.0,
        ema_regression_penalty: DEFAULT_EMA_REGRESSION_PENALTY,
        cache_affinity_mode,
        affinity_bonus,
        auto_tune_enabled: auto_tune_enabled(id),
        learning: None,
    };
    enforce_priority_policy(&mut profile);
    profile
}

fn built_in_affinity(id: RoutingProfileId) -> (RoutingCacheAffinityMode, f64) {
    match id {
        RoutingProfileId::CacheAffinityPlus => (RoutingCacheAffinityMode::PreferCached, 6.0),
        RoutingProfileId::FirstByte => (RoutingCacheAffinityMode::ScoreBonus, 3.0),
        RoutingProfileId::Balanced | RoutingProfileId::Custom => (RoutingCacheAffinityMode::ScoreBonus, 2.0),
        RoutingProfileId::CostOptimal | RoutingProfileId::HighAvailability | RoutingProfileId::HighTps | RoutingProfileId::FixedPriorityPlus => {
            (RoutingCacheAffinityMode::Disabled, 0.0)
        }
    }
}

fn built_in_definition(id: RoutingProfileId) -> (&'static str, &'static str, RoutingProfileWeights) {
    match id {
        RoutingProfileId::Balanced => (
            "Balanced",
            "Success, latency, cost, and headroom balanced.",
            weights(0.28, 0.19, 0.17, 0.09, 0.15, 0.12, 0.0),
        ),
        RoutingProfileId::FirstByte => (
            "First Byte",
            "Prioritizes p90 first-byte time for interactive streams.",
            weights(0.26, 0.36, 0.08, 0.04, 0.06, 0.20, 0.0),
        ),
        RoutingProfileId::HighTps => (
            "High TPS",
            "Prioritizes output throughput and capacity headroom.",
            weights(0.19, 0.09, 0.09, 0.34, 0.06, 0.23, 0.0),
        ),
        RoutingProfileId::CostOptimal => (
            "Cost Optimal",
            "Minimizes estimated upstream cost with a success floor.",
            weights(0.26, 0.08, 0.08, 0.04, 0.46, 0.08, 0.0),
        ),
        RoutingProfileId::HighAvailability => (
            "High Availability",
            "Prioritizes success rate and low error risk.",
            weights(0.44, 0.13, 0.19, 0.04, 0.04, 0.16, 0.0),
        ),
        RoutingProfileId::CacheAffinityPlus => (
            "Cache Affinity Plus",
            "Extends cache affinity with health and TTFB scoring.",
            weights(0.28, 0.30, 0.12, 0.04, 0.10, 0.16, 0.0),
        ),
        RoutingProfileId::FixedPriorityPlus => (
            "Fixed Priority Plus",
            "Keeps administrator priority while excluding unhealthy routes.",
            weights(0.18, 0.08, 0.08, 0.04, 0.08, 0.10, 0.44),
        ),
        RoutingProfileId::Custom => (
            "Custom",
            "Administrator-controlled routing profile.",
            weights(0.27, 0.16, 0.16, 0.11, 0.16, 0.14, 0.0),
        ),
    }
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

fn upsert_profile_item(profiles: &mut Vec<RoutingProfile>, overlay: RoutingProfile) {
    let mut by_id = profiles
        .iter()
        .enumerate()
        .map(|(index, profile)| (profile.id.as_str(), index))
        .collect::<HashMap<_, _>>();
    if let Some(index) = by_id.remove(overlay.id.as_str()) {
        profiles[index] = overlay;
    } else {
        profiles.push(overlay);
    }
}

fn new_version() -> String {
    format!("profile-{}", uuid::Uuid::now_v7())
}

fn enforce_priority_policy(profile: &mut RoutingProfile) {
    if profile.id != RoutingProfileId::FixedPriorityPlus {
        profile.weights.priority = 0.0;
    }
}

fn auto_tune_enabled(id: RoutingProfileId) -> bool {
    let _ = id;
    true
}

fn validate_profile(profile: &RoutingProfile) -> Result<(), LlmProxyError> {
    validate_weights(profile)?;
    validate_score_parameter("exploration_k", profile.exploration_k)?;
    validate_percent("exploration_budget_percent", profile.exploration_budget_percent)?;
    validate_score_parameter("conversion_penalty", profile.conversion_penalty)?;
    validate_score_parameter("stale_metric_penalty", profile.stale_metric_penalty)?;
    validate_percent("ema_regression_penalty", profile.ema_regression_penalty)?;
    validate_score_parameter("affinity_bonus", profile.affinity_bonus)
}

fn validate_weights(profile: &RoutingProfile) -> Result<(), LlmProxyError> {
    let weights = [
        ("success", profile.weights.success),
        ("ttfb", profile.weights.ttfb),
        ("latency", profile.weights.latency),
        ("tps", profile.weights.tps),
        ("cost", profile.weights.cost),
        ("headroom", profile.weights.headroom),
        ("priority", profile.weights.priority),
    ];
    for (name, value) in weights {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(LlmProxyError::InvalidRequest(format!("routing profile weight {name} must be between 0 and 1")));
        }
    }
    let total = weights.iter().map(|(_, value)| *value).sum::<f64>();
    if (total - PROFILE_WEIGHT_SUM).abs() > WEIGHT_SUM_TOLERANCE {
        return Err(LlmProxyError::InvalidRequest(format!(
            "routing profile weights must sum to 1.0; got {total:.3}"
        )));
    }
    Ok(())
}

fn validate_percent(name: &str, value: f64) -> Result<(), LlmProxyError> {
    if value.is_finite() && (PERCENT_MIN..=PERCENT_MAX).contains(&value) {
        return Ok(());
    }
    Err(LlmProxyError::InvalidRequest(format!("{name} must be between 0 and 100")))
}

fn validate_score_parameter(name: &str, value: f64) -> Result<(), LlmProxyError> {
    if value.is_finite() && value >= 0.0 {
        return Ok(());
    }
    Err(LlmProxyError::InvalidRequest(format!("{name} must be a finite non-negative number")))
}

#[cfg(test)]
pub(super) fn test_only_builtin_profile(id: RoutingProfileId) -> RoutingProfile {
    built_in_profile(id)
}

#[cfg(test)]
mod tests {
    use types::provider::{DEFAULT_EMA_REGRESSION_PENALTY, DEFAULT_EXPLORATION_BUDGET_PERCENT, RoutingCacheAffinityMode};

    use super::*;

    #[test]
    fn builtin_profiles_have_strategy_defaults() {
        let profile = built_in_profile(RoutingProfileId::Balanced);

        assert_eq!(profile.exploration_budget_percent, DEFAULT_EXPLORATION_BUDGET_PERCENT);
        assert_eq!(profile.ema_regression_penalty, DEFAULT_EMA_REGRESSION_PENALTY);
        assert_eq!(profile.cache_affinity_mode, RoutingCacheAffinityMode::ScoreBonus);
        assert_eq!(profile.affinity_bonus, 2.0);
        assert_eq!(
            built_in_profile(RoutingProfileId::FirstByte).cache_affinity_mode,
            RoutingCacheAffinityMode::ScoreBonus
        );
        assert_eq!(built_in_profile(RoutingProfileId::FirstByte).affinity_bonus, 3.0);
        assert_eq!(
            built_in_profile(RoutingProfileId::CacheAffinityPlus).cache_affinity_mode,
            RoutingCacheAffinityMode::PreferCached
        );
        assert_eq!(
            built_in_profile(RoutingProfileId::CostOptimal).cache_affinity_mode,
            RoutingCacheAffinityMode::Disabled
        );
        assert_eq!(built_in_profile(RoutingProfileId::CostOptimal).affinity_bonus, 0.0);
    }

    #[test]
    fn validation_rejects_invalid_weight_total() {
        let mut profile = built_in_profile(RoutingProfileId::Balanced);
        profile.weights.cost = 0.30;

        let error = validate_profile(&profile).expect_err("invalid weight total should fail");

        assert!(error.to_string().contains("weights must sum"));
    }

    #[test]
    fn validation_rejects_out_of_range_exploration_budget() {
        let mut profile = built_in_profile(RoutingProfileId::Balanced);
        profile.exploration_budget_percent = 120.0;

        let error = validate_profile(&profile).expect_err("invalid exploration budget should fail");

        assert!(error.to_string().contains("exploration_budget_percent"));
    }

    #[test]
    fn validation_rejects_negative_ema_penalty() {
        let mut profile = built_in_profile(RoutingProfileId::Balanced);
        profile.ema_regression_penalty = -1.0;

        let error = validate_profile(&profile).expect_err("negative EMA penalty should fail");

        assert!(error.to_string().contains("ema_regression_penalty"));
    }
}
