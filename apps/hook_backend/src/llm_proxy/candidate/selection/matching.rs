use std::collections::HashSet;

use types::provider::{ProviderSchedulingMode, provider_key_time_range_contains};

use super::{
    CandidateParts,
    matching_order::{OrderedKeysInput, current_utc_minute, ordered_keys, promote_affinity_endpoint},
};
use crate::llm_proxy::{
    AffinitySelection,
    cache::snapshot::{CachedBillingGroup, CachedEndpoint, CachedModelBinding, CachedProvider, CachedProviderKey, CachedUserAccess, SchedulingSnapshot},
    candidate::CandidateRequest,
    capabilities::capability_list_enabled,
    formats,
    model_access::provider_allowed,
};

pub(super) struct MatchingCandidatePartsInput<'a> {
    pub(super) snapshot: &'a SchedulingSnapshot,
    pub(super) group: &'a CachedBillingGroup,
    pub(super) user_access: Option<&'a CachedUserAccess>,
    pub(super) model_id: &'a str,
    pub(super) request: CandidateRequest<'a>,
    pub(super) affinity: Option<&'a AffinitySelection>,
    pub(super) scheduling_mode: ProviderSchedulingMode,
    pub(super) request_id: &'a str,
    pub(super) cooled_provider_ids: &'a HashSet<String>,
}

pub(super) fn matching_candidate_parts(input: MatchingCandidatePartsInput<'_>) -> Vec<CandidateParts> {
    matching_candidate_parts_at(input, current_utc_minute())
}

pub(super) fn matching_candidate_parts_at(input: MatchingCandidatePartsInput<'_>, current_minute: u16) -> Vec<CandidateParts> {
    let mut candidates = Vec::new();
    for provider in input
        .snapshot
        .providers
        .iter()
        .filter(|provider| !input.cooled_provider_ids.contains(&provider.id))
        .filter(|provider| provider_allowed(input.group, input.user_access, provider))
    {
        append_provider_candidate(
            AppendProviderCandidateInput {
                snapshot: input.snapshot,
                provider,
                group: input.group,
                model_id: input.model_id,
                request: &input.request,
                affinity: input.affinity,
                scheduling_mode: input.scheduling_mode,
                request_id: input.request_id,
                current_minute,
            },
            &mut candidates,
        );
    }
    candidates
}

struct AppendProviderCandidateInput<'a> {
    snapshot: &'a SchedulingSnapshot,
    provider: &'a CachedProvider,
    group: &'a CachedBillingGroup,
    model_id: &'a str,
    request: &'a CandidateRequest<'a>,
    affinity: Option<&'a AffinitySelection>,
    scheduling_mode: ProviderSchedulingMode,
    request_id: &'a str,
    current_minute: u16,
}

fn append_provider_candidate(input: AppendProviderCandidateInput<'_>, output: &mut Vec<CandidateParts>) {
    let Some(model) = provider_model(input.provider, input.model_id) else {
        return;
    };
    if !global_model_supports_required_capability(input.request, input.snapshot, &model.global_model_id) {
        return;
    }
    let affinity = matching_affinity(input.provider, input.affinity);
    let endpoints = ordered_endpoints(input.provider, input.group, input.model_id, input.request, input.current_minute, affinity);
    let allowed_keys = allowed_keys(input.provider, input.group, input.current_minute);
    let keys = ordered_keys(OrderedKeysInput {
        keys: &allowed_keys,
        affinity,
        scheduling_mode: input.scheduling_mode,
        request_id: input.request_id,
    });
    if endpoints.is_empty() || keys.is_empty() {
        return;
    }
    append_key_candidates(input.provider, model, input.model_id, input.request, endpoints, keys, output);
}

fn append_key_candidates(
    provider: &CachedProvider,
    model: CachedModelBinding,
    model_id: &str,
    request: &CandidateRequest<'_>,
    endpoints: Vec<CachedEndpoint>,
    keys: Vec<CachedProviderKey>,
    output: &mut Vec<CandidateParts>,
) {
    for key in keys {
        let key_endpoints = endpoints
            .iter()
            .filter(|endpoint| key_allows_candidate(&key, model_id, endpoint, request))
            .cloned()
            .collect::<Vec<_>>();
        if key_endpoints.is_empty() {
            continue;
        }
        output.push(CandidateParts {
            provider: provider.clone(),
            endpoints: key_endpoints,
            keys: vec![key],
            model: model.clone(),
            client_api_format: request.api_format.to_owned(),
            routing_api_format: request.routing_api_format.to_owned(),
            is_cached: false,
        });
    }
}

fn ordered_endpoints(
    provider: &CachedProvider,
    group: &CachedBillingGroup,
    model_id: &str,
    request: &CandidateRequest<'_>,
    current_minute: u16,
    affinity: Option<&AffinitySelection>,
) -> Vec<CachedEndpoint> {
    let (mut exact, converted): (Vec<_>, Vec<_>) = provider
        .endpoints
        .iter()
        .filter(|endpoint| endpoint_allowed(provider, endpoint, request))
        .filter(|endpoint| {
            provider
                .keys
                .iter()
                .any(|key| key_allowed_for_model_endpoint(key, model_id, endpoint, group, current_minute))
        })
        .cloned()
        .partition(|endpoint| endpoint_exact(endpoint, request));
    exact.extend(converted);
    promote_affinity_endpoint(&mut exact, affinity, |endpoint| endpoint.id.as_str());
    exact
}

fn allowed_keys(provider: &CachedProvider, group: &CachedBillingGroup, current_minute: u16) -> Vec<CachedProviderKey> {
    provider.keys.iter().filter(|key| key_allowed(key, group, current_minute)).cloned().collect()
}

fn provider_model(provider: &CachedProvider, model_id: &str) -> Option<CachedModelBinding> {
    provider
        .models
        .iter()
        .find(|model| model.global_model_id == model_id && model.is_active)
        .map(selected_provider_model)
}

fn endpoint_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: &CandidateRequest<'_>) -> bool {
    endpoint.is_active && (endpoint_exact(endpoint, request) || conversion_allowed(provider, endpoint, request))
}

fn conversion_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: &CandidateRequest<'_>) -> bool {
    if request.has_openai_responses_custom_tool_items {
        return false;
    }
    (provider.enable_format_conversion || endpoint_accepts_conversion(endpoint))
        && formats::formats_compatible(request.routing_api_format, &endpoint.api_format, request.is_stream)
        && !endpoint_exact(endpoint, request)
}

fn endpoint_exact(endpoint: &CachedEndpoint, request: &CandidateRequest<'_>) -> bool {
    formats::formats_exact(request.routing_api_format, &endpoint.api_format, request.is_stream).unwrap_or(false)
}

fn endpoint_accepts_conversion(endpoint: &CachedEndpoint) -> bool {
    endpoint
        .format_acceptance_config
        .as_ref()
        .and_then(|value| value.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn key_allowed(key: &CachedProviderKey, group: &CachedBillingGroup, current_minute: u16) -> bool {
    key.is_active && !key.api_formats.is_empty() && key_in_group_scope(key, group) && key_time_range_allowed(key, current_minute)
}

fn key_in_group_scope(key: &CachedProviderKey, group: &CachedBillingGroup) -> bool {
    group.allowed_provider_key_ids.as_ref().is_none_or(|ids| ids.iter().any(|id| id == &key.id))
}

fn key_allowed_for_model_endpoint(key: &CachedProviderKey, model_id: &str, endpoint: &CachedEndpoint, group: &CachedBillingGroup, current_minute: u16) -> bool {
    key_allowed(key, group, current_minute) && key_allows_model(key, model_id) && key.api_formats.iter().any(|api_format| api_format == &endpoint.api_format)
}

fn key_allows_candidate(key: &CachedProviderKey, model_id: &str, endpoint: &CachedEndpoint, request: &CandidateRequest<'_>) -> bool {
    key_allows_model(key, model_id) && key_allows_endpoint(key, endpoint) && key_supports_required_capability(key, request.required_capability)
}

fn key_allows_endpoint(key: &CachedProviderKey, endpoint: &CachedEndpoint) -> bool {
    key.api_formats.iter().any(|api_format| api_format == &endpoint.api_format)
}

fn global_model_supports_required_capability(request: &CandidateRequest<'_>, snapshot: &SchedulingSnapshot, model_id: &str) -> bool {
    let Some(required) = request.required_capability else {
        return true;
    };
    snapshot
        .models
        .iter()
        .find(|model| model.id == model_id)
        .is_some_and(|model| capability_list_enabled(model.supported_capabilities.as_deref(), required))
}

fn key_time_range_allowed(key: &CachedProviderKey, current_minute: u16) -> bool {
    if !key.time_range_enabled {
        return true;
    }
    let (Some(start), Some(end)) = (key.time_range_start_minute, key.time_range_end_minute) else {
        return false;
    };
    provider_key_time_range_contains(current_minute, start, end)
}

fn key_allows_model(key: &CachedProviderKey, model_id: &str) -> bool {
    key.allowed_model_ids.is_empty() || key.allowed_model_ids.iter().any(|id| id == model_id)
}

fn key_supports_required_capability(key: &CachedProviderKey, required: Option<&str>) -> bool {
    let Some(required) = required.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };
    required != "image_generation" || key.supports_image_generation
}

fn selected_provider_model(model: &CachedModelBinding) -> CachedModelBinding {
    let mut selected = model.clone();
    selected.provider_model_name = selected_provider_model_name(model);
    selected
}

fn matching_affinity<'a>(provider: &CachedProvider, affinity: Option<&'a AffinitySelection>) -> Option<&'a AffinitySelection> {
    affinity.filter(|record| record.provider_id == provider.id)
}

fn selected_provider_model_name(model: &CachedModelBinding) -> String {
    model
        .provider_model_mapping
        .as_ref()
        .map(|mapping| mapping.name.clone())
        .unwrap_or_else(|| model.provider_model_name.clone())
}
