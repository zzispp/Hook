use std::collections::{HashMap, HashSet};

use proxy::scheduler::{AffinityCandidate, Candidate, CandidateBuilder, ModelAccessPolicy, PriorityMode, ProviderSnapshot, SchedulerInput, SchedulingMode};
use types::{
    api_token::{ApiToken, ModelAccessMode},
    provider::{ProviderPriorityMode, ProviderSchedulingMode},
};

use super::{CandidatePartKey, CandidateParts};
use crate::llm_proxy::{
    AffinitySelection, LlmProxyError,
    cache::snapshot::{CachedBillingGroup, CachedProvider, CachedUserAccess},
    candidate::CandidateRequest,
    formats,
};

pub(super) struct OrderCandidatePartsInput<'a> {
    pub(super) parts: Vec<CandidateParts>,
    pub(super) token: &'a ApiToken,
    pub(super) group: &'a CachedBillingGroup,
    pub(super) user_access: Option<&'a CachedUserAccess>,
    pub(super) request: CandidateRequest<'a>,
    pub(super) model_id: &'a str,
    pub(super) request_id: &'a str,
    pub(super) affinity: Option<AffinitySelection>,
    pub(super) mode: ProviderSchedulingMode,
    pub(super) priority_mode: ProviderPriorityMode,
}

pub(super) fn order_candidate_parts(input: OrderCandidatePartsInput<'_>) -> Result<Vec<CandidateParts>, LlmProxyError> {
    let OrderCandidatePartsInput {
        parts,
        token,
        group,
        user_access,
        request,
        model_id,
        request_id,
        affinity,
        mode,
        priority_mode,
    } = input;
    let input = scheduler_input(SchedulerInputArgs {
        parts: &parts,
        token,
        group,
        user_access,
        request,
        model_id,
        request_id,
        affinity: affinity.as_ref(),
        mode,
        priority_mode,
    })?;
    let mut by_key = parts.into_iter().map(|part| (part_key(&part), part)).collect::<HashMap<_, _>>();
    let mut candidates = by_key
        .values()
        .map(|part| scheduler_candidate(part, priority_mode))
        .collect::<Result<Vec<_>, _>>()?;
    CandidateBuilder::order(&mut candidates, &input);
    Ok(candidates.into_iter().filter_map(|candidate| ordered_part(&mut by_key, candidate)).collect())
}

fn scheduler_input(args: SchedulerInputArgs<'_>) -> Result<SchedulerInput, LlmProxyError> {
    let client_format = formats::endpoint_metadata(args.request.routing_api_format, args.request.is_stream)?.data_format;
    Ok(SchedulerInput {
        group_code: args.group.code.clone(),
        group_is_active: args.group.is_active,
        group_allowed_model_ids: args.group.allowed_model_ids.clone(),
        group_allowed_provider_key_ids: args.group.allowed_provider_key_ids.clone(),
        user_allowed_model_ids: args.user_access.map(|access| access.allowed_model_ids.clone()).unwrap_or_default(),
        user_allowed_provider_ids: args.user_access.map(|access| access.allowed_provider_ids.clone()).unwrap_or_default(),
        token_model_policy: token_model_policy(args.token),
        requested_model_id: args.model_id.to_owned(),
        client_format,
        is_stream: args.request.is_stream,
        affinity: args.affinity.map(proxy_affinity),
        load_balance_seed: Some(args.request_id.to_owned()),
        scheduling_mode: scheduler_mode(args.mode),
        priority_mode: priority_mode(args.priority_mode),
        global_keep_priority_on_conversion: false,
        global_format_conversion_enabled: true,
        providers: scheduler_providers(args.parts),
    })
}

struct SchedulerInputArgs<'a> {
    parts: &'a [CandidateParts],
    token: &'a ApiToken,
    group: &'a CachedBillingGroup,
    user_access: Option<&'a CachedUserAccess>,
    request: CandidateRequest<'a>,
    model_id: &'a str,
    request_id: &'a str,
    affinity: Option<&'a AffinitySelection>,
    mode: ProviderSchedulingMode,
    priority_mode: ProviderPriorityMode,
}

fn scheduler_candidate(parts: &CandidateParts, priority_mode: ProviderPriorityMode) -> Result<Candidate, LlmProxyError> {
    let endpoint = primary_endpoint(parts);
    let key = primary_key(parts);
    let provider_api_format = formats::endpoint_metadata(&endpoint.api_format, false)?.data_format;
    let needs_conversion = formats::needs_conversion(&parts.routing_api_format, &endpoint.api_format, false)?;
    Ok(Candidate {
        provider_id: parts.provider.id.clone(),
        provider_name: parts.provider.name.clone(),
        endpoint_id: endpoint.id.clone(),
        key_id: key.id.clone(),
        global_model_id: parts.model.global_model_id.clone(),
        provider_model_name: parts.model.provider_model_name.clone(),
        provider_api_format,
        needs_conversion,
        is_cached: parts.is_cached,
        provider_priority: parts.provider.priority,
        key_priority: match priority_mode {
            ProviderPriorityMode::Provider => key.internal_priority,
            ProviderPriorityMode::Key => key
                .global_priority_by_format
                .get(&endpoint.api_format)
                .copied()
                .unwrap_or(key.internal_priority),
        },
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

fn priority_mode(mode: ProviderPriorityMode) -> PriorityMode {
    match mode {
        ProviderPriorityMode::Provider => PriorityMode::Provider,
        ProviderPriorityMode::Key => PriorityMode::Key,
    }
}

fn part_key(parts: &CandidateParts) -> CandidatePartKey {
    (parts.provider.id.clone(), primary_endpoint(parts).id.clone(), primary_key(parts).id.clone())
}

fn candidate_key(candidate: &Candidate) -> CandidatePartKey {
    (candidate.provider_id.clone(), candidate.endpoint_id.clone(), candidate.key_id.clone())
}

fn ordered_part(by_key: &mut HashMap<CandidatePartKey, CandidateParts>, candidate: Candidate) -> Option<CandidateParts> {
    let mut part = by_key.remove(&candidate_key(&candidate))?;
    part.is_cached = candidate.is_cached;
    Some(part)
}

fn proxy_affinity(affinity: &AffinitySelection) -> AffinityCandidate {
    AffinityCandidate {
        provider_id: affinity.provider_id.clone(),
        endpoint_id: affinity.endpoint_id.clone(),
        key_id: affinity.key_id.clone(),
    }
}

fn primary_endpoint(parts: &CandidateParts) -> &crate::llm_proxy::cache::snapshot::CachedEndpoint {
    &parts.endpoints[0]
}

fn primary_key(parts: &CandidateParts) -> &crate::llm_proxy::cache::snapshot::CachedProviderKey {
    let endpoint = primary_endpoint(parts);
    parts
        .keys
        .iter()
        .find(|key| key.api_formats.iter().any(|format| format == &endpoint.api_format))
        .expect("candidate parts must contain at least one key for the primary endpoint")
}
