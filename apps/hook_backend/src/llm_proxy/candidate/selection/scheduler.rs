use std::collections::{HashMap, HashSet};

use proxy::scheduler::{Candidate, CandidateBuilder, ModelAccessPolicy, ProviderSnapshot, SchedulerInput, SchedulingMode};
use types::{
    api_token::{ApiToken, ModelAccessMode},
    provider::ProviderSchedulingMode,
};

use super::{CandidatePartKey, CandidateParts};
use crate::llm_proxy::{
    LlmProxyError,
    cache::snapshot::{CachedBillingGroup, CachedProvider},
    candidate::CandidateRequest,
    formats,
};

pub(super) fn order_candidate_parts(
    parts: Vec<CandidateParts>,
    token: &ApiToken,
    group: &CachedBillingGroup,
    request: CandidateRequest<'_>,
    model_id: &str,
    request_id: &str,
    affinity_key: Option<String>,
    mode: ProviderSchedulingMode,
) -> Result<Vec<CandidateParts>, LlmProxyError> {
    let input = scheduler_input(&parts, token, group, request, model_id, request_id, affinity_key, mode)?;
    let mut by_key = parts.into_iter().map(|part| (part_key(&part), part)).collect::<HashMap<_, _>>();
    let mut candidates = by_key.values().map(scheduler_candidate).collect::<Result<Vec<_>, _>>()?;
    CandidateBuilder::order(&mut candidates, &input);
    Ok(candidates
        .into_iter()
        .filter_map(|candidate| by_key.remove(&candidate_key(&candidate)))
        .collect())
}

fn scheduler_input(
    parts: &[CandidateParts],
    token: &ApiToken,
    group: &CachedBillingGroup,
    request: CandidateRequest<'_>,
    model_id: &str,
    request_id: &str,
    affinity_key: Option<String>,
    mode: ProviderSchedulingMode,
) -> Result<SchedulerInput, LlmProxyError> {
    Ok(SchedulerInput {
        group_code: group.code.clone(),
        group_is_active: group.is_active,
        group_allowed_model_ids: group.allowed_model_ids.clone(),
        group_allowed_provider_ids: group.allowed_provider_ids.clone(),
        token_model_policy: token_model_policy(token),
        requested_model_id: model_id.to_owned(),
        client_format: formats::parse_api_format(request.api_format)?,
        is_stream: request.is_stream,
        affinity_key,
        load_balance_seed: Some(request_id.to_owned()),
        scheduling_mode: scheduler_mode(mode),
        global_keep_priority_on_conversion: false,
        global_format_conversion_enabled: true,
        providers: scheduler_providers(parts),
    })
}

fn scheduler_candidate(parts: &CandidateParts) -> Result<Candidate, LlmProxyError> {
    Ok(Candidate {
        provider_id: parts.provider.id.clone(),
        provider_name: parts.provider.name.clone(),
        endpoint_id: parts.endpoint.id.clone(),
        key_id: parts.key.id.clone(),
        global_model_id: parts.model.global_model_id.clone(),
        provider_model_name: parts.model.provider_model_name.clone(),
        provider_api_format: formats::parse_api_format(&parts.endpoint.api_format)?,
        needs_conversion: parts.needs_conversion,
        is_cached: false,
        provider_priority: parts.provider.priority,
        key_priority: parts.key.internal_priority,
    })
}

fn scheduler_providers(parts: &[CandidateParts]) -> Vec<ProviderSnapshot> {
    let mut seen = HashSet::new();
    parts
        .iter()
        .filter(|part| seen.insert(part.provider.id.clone()))
        .map(|part| provider_snapshot(&part.provider))
        .collect()
}

fn provider_snapshot(provider: &CachedProvider) -> ProviderSnapshot {
    ProviderSnapshot {
        id: provider.id.clone(),
        name: provider.name.clone(),
        priority: provider.priority,
        keep_priority_on_conversion: provider.keep_priority_on_conversion,
        enable_format_conversion: provider.enable_format_conversion,
        is_active: provider.is_active,
        endpoints: Vec::new(),
        keys: Vec::new(),
        models: Vec::new(),
    }
}

fn token_model_policy(token: &ApiToken) -> ModelAccessPolicy {
    match token.model_access_mode {
        ModelAccessMode::All => ModelAccessPolicy::All,
        ModelAccessMode::Limited => ModelAccessPolicy::Limited(token.allowed_model_ids.clone()),
    }
}

fn scheduler_mode(mode: ProviderSchedulingMode) -> SchedulingMode {
    match mode {
        ProviderSchedulingMode::FixedOrder => SchedulingMode::FixedOrder,
        ProviderSchedulingMode::CacheAffinity => SchedulingMode::CacheAffinity,
        ProviderSchedulingMode::LoadBalance => SchedulingMode::LoadBalance,
    }
}

fn part_key(parts: &CandidateParts) -> CandidatePartKey {
    (
        parts.provider.id.clone(),
        parts.endpoint.id.clone(),
        parts.key.id.clone(),
        parts.model.global_model_id.clone(),
    )
}

fn candidate_key(candidate: &Candidate) -> CandidatePartKey {
    (
        candidate.provider_id.clone(),
        candidate.endpoint_id.clone(),
        candidate.key_id.clone(),
        candidate.global_model_id.clone(),
    )
}
