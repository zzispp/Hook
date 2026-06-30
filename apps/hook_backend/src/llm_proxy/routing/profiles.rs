use std::collections::HashMap;

use storage::provider::ProviderStore;
use types::provider::{RoutingProfile, RoutingProfileId, RoutingProfileUpsert, RoutingProfileWeights};

use crate::llm_proxy::{LlmProxyError, LlmProxyState};

use super::learning::apply_profile_learning;

const BUILTIN_PROFILE_VERSION: &str = "builtin-v2";
const WEIGHT_SUM_TOLERANCE: f64 = 0.001;

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
    validate_profile_upsert(id, &patch)?;
    let mut profile = raw_profiles(state)
        .await?
        .into_iter()
        .find(|profile| profile.id == id)
        .unwrap_or_else(|| built_in_profile(id));
    apply_patch(&mut profile, patch);
    enforce_priority_policy(&mut profile);
    profile.learning = None;
    profile.version = new_version();
    ProviderStore::new(state.database.clone())
        .upsert_routing_profile_overlay(profile)
        .await
        .map_err(LlmProxyError::from)
}

fn validate_profile_upsert(id: RoutingProfileId, patch: &RoutingProfileUpsert) -> Result<(), LlmProxyError> {
    if let Some(weights) = patch.weights.as_ref() {
        validate_weights(id, weights)?;
    }
    if patch.min_samples == Some(0) {
        return invalid_profile_patch("min_samples must be at least 1");
    }
    validate_non_negative("exploration_k", patch.exploration_k)?;
    validate_non_negative("conversion_penalty", patch.conversion_penalty)?;
    validate_non_negative("stale_metric_penalty", patch.stale_metric_penalty)?;
    validate_non_negative("affinity_bonus", patch.affinity_bonus)?;
    validate_unit_interval("ema_alpha", patch.ema_alpha)?;
    validate_non_negative_i64("ema_max_freshness_seconds", patch.ema_max_freshness_seconds)?;
    validate_non_negative("ema_recent_weight", patch.ema_recent_weight)?;
    validate_non_negative("ema_recent_cap", patch.ema_recent_cap)?;
    validate_non_negative("exploration_weight", patch.exploration_weight)?;
    validate_non_negative("exploration_cap", patch.exploration_cap)?;
    validate_percentage_score("exploration_min_success_score", patch.exploration_min_success_score)
}

fn validate_weights(id: RoutingProfileId, weights: &RoutingProfileWeights) -> Result<(), LlmProxyError> {
    for (name, value) in weight_values(weights) {
        validate_finite_non_negative(name, value)?;
    }
    let sum = weight_values(weights).iter().map(|(_, value)| value).sum::<f64>();
    if (sum - 1.0).abs() > WEIGHT_SUM_TOLERANCE {
        return invalid_profile_patch("routing profile weights must sum to 1");
    }
    if id != RoutingProfileId::FixedPriorityPlus && weights.priority > f64::EPSILON {
        return invalid_profile_patch("priority weight is only allowed for fixed_priority_plus");
    }
    Ok(())
}

fn validate_non_negative(name: &str, value: Option<f64>) -> Result<(), LlmProxyError> {
    if let Some(value) = value {
        validate_finite_non_negative(name, value)?;
    }
    Ok(())
}

fn validate_non_negative_i64(name: &str, value: Option<i64>) -> Result<(), LlmProxyError> {
    if let Some(value) = value
        && value < 0
    {
        return invalid_profile_patch(format!("{name} must be non-negative"));
    }
    Ok(())
}

fn validate_unit_interval(name: &str, value: Option<f64>) -> Result<(), LlmProxyError> {
    if let Some(value) = value
        && (!value.is_finite() || !(0.0..=1.0).contains(&value))
    {
        return invalid_profile_patch(format!("{name} must be between 0 and 1"));
    }
    Ok(())
}

fn validate_percentage_score(name: &str, value: Option<f64>) -> Result<(), LlmProxyError> {
    if let Some(value) = value
        && (!value.is_finite() || !(0.0..=100.0).contains(&value))
    {
        return invalid_profile_patch(format!("{name} must be between 0 and 100"));
    }
    Ok(())
}

fn validate_finite_non_negative(name: &str, value: f64) -> Result<(), LlmProxyError> {
    if !value.is_finite() || value < 0.0 {
        return invalid_profile_patch(format!("{name} must be finite and non-negative"));
    }
    Ok(())
}

fn weight_values(weights: &RoutingProfileWeights) -> [(&'static str, f64); 7] {
    [
        ("success", weights.success),
        ("first_token", weights.first_token),
        ("latency", weights.latency),
        ("tps", weights.tps),
        ("cost", weights.cost),
        ("headroom", weights.headroom),
        ("priority", weights.priority),
    ]
}

fn invalid_profile_patch(message: impl Into<String>) -> Result<(), LlmProxyError> {
    Err(LlmProxyError::InvalidRequest(format!("invalid routing profile patch: {}", message.into())))
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
    if let Some(value) = patch.conversion_penalty {
        profile.conversion_penalty = value;
    }
    if let Some(value) = patch.stale_metric_penalty {
        profile.stale_metric_penalty = value;
    }
    if let Some(value) = patch.affinity_bonus {
        profile.affinity_bonus = value;
    }
    if let Some(value) = patch.prior_sample_cap {
        profile.prior_sample_cap = value;
    }
    if let Some(value) = patch.contextual_exploration_enabled {
        profile.contextual_exploration_enabled = value;
    }
    if let Some(value) = patch.ema_alpha {
        profile.ema_alpha = value;
    }
    if let Some(value) = patch.ema_max_freshness_seconds {
        profile.ema_max_freshness_seconds = value;
    }
    if let Some(value) = patch.ema_recent_weight {
        profile.ema_recent_weight = value;
    }
    if let Some(value) = patch.ema_recent_cap {
        profile.ema_recent_cap = value;
    }
    if let Some(value) = patch.exploration_weight {
        profile.exploration_weight = value;
    }
    if let Some(value) = patch.exploration_cap {
        profile.exploration_cap = value;
    }
    if let Some(value) = patch.exploration_min_success_score {
        profile.exploration_min_success_score = value;
    }
    if let Some(value) = patch.auto_tune_enabled {
        profile.auto_tune_enabled = value;
    }
}

fn built_in_profiles() -> Vec<RoutingProfile> {
    [
        built_in_profile(RoutingProfileId::Balanced),
        built_in_profile(RoutingProfileId::FirstToken),
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
    let mut profile = RoutingProfile {
        id,
        name: name.into(),
        description: description.into(),
        weights,
        version: BUILTIN_PROFILE_VERSION.into(),
        min_samples: 12,
        exploration_k: 4.5,
        conversion_penalty: 6.0,
        stale_metric_penalty: 8.0,
        affinity_bonus: 6.0,
        prior_sample_cap: types::provider::default_prior_sample_cap(),
        contextual_exploration_enabled: types::provider::default_contextual_exploration_enabled(),
        ema_alpha: types::provider::default_ema_alpha(),
        ema_max_freshness_seconds: types::provider::default_ema_max_freshness_seconds(),
        ema_recent_weight: types::provider::default_ema_recent_weight(),
        ema_recent_cap: types::provider::default_ema_recent_cap(),
        exploration_weight: types::provider::default_exploration_weight(),
        exploration_cap: types::provider::default_exploration_cap(),
        exploration_min_success_score: types::provider::default_exploration_min_success_score(),
        auto_tune_enabled: auto_tune_enabled(id),
        learning: None,
    };
    enforce_priority_policy(&mut profile);
    profile
}

fn built_in_definition(id: RoutingProfileId) -> (&'static str, &'static str, RoutingProfileWeights) {
    match id {
        RoutingProfileId::Balanced => (
            "Balanced",
            "Success, latency, cost, and headroom balanced.",
            weights(0.28, 0.19, 0.17, 0.09, 0.15, 0.12, 0.0),
        ),
        RoutingProfileId::FirstToken => (
            "First Token",
            "Prioritizes p90 first-token time for interactive streams.",
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
            "Extends cache affinity with health and first-token scoring.",
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

fn weights(success: f64, first_token: f64, latency: f64, tps: f64, cost: f64, headroom: f64, priority: f64) -> RoutingProfileWeights {
    RoutingProfileWeights {
        success,
        first_token,
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

#[cfg(test)]
pub(super) fn test_only_builtin_profile(id: RoutingProfileId) -> RoutingProfile {
    built_in_profile(id)
}

#[cfg(test)]
#[path = "profiles/tests.rs"]
mod tests;
