use std::collections::HashSet;

use time::OffsetDateTime;
use types::provider::{ProviderSchedulingMode, provider_key_minute_of_day, provider_key_time_range_contains};

use super::CandidateParts;
use crate::llm_proxy::{
    AffinitySelection,
    cache::snapshot::{CachedBillingGroup, CachedEndpoint, CachedModelBinding, CachedProvider, CachedProviderKey, CachedUserAccess, SchedulingSnapshot},
    candidate::CandidateRequest,
    formats,
    model_access::provider_allowed,
};

const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

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
                request: input.request,
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
    request: CandidateRequest<'a>,
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
    let keys = ordered_keys(OrderedKeysInput {
        provider: input.provider,
        group: input.group,
        affinity,
        scheduling_mode: input.scheduling_mode,
        request_id: input.request_id,
        current_minute: input.current_minute,
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
    request: CandidateRequest<'_>,
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

struct OrderedKeysInput<'a> {
    provider: &'a CachedProvider,
    group: &'a CachedBillingGroup,
    affinity: Option<&'a AffinitySelection>,
    scheduling_mode: ProviderSchedulingMode,
    request_id: &'a str,
    current_minute: u16,
}

fn ordered_endpoints(
    provider: &CachedProvider,
    group: &CachedBillingGroup,
    model_id: &str,
    request: CandidateRequest<'_>,
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
    promote_affinity_endpoint(&mut exact, affinity);
    exact
}

fn ordered_keys(input: OrderedKeysInput<'_>) -> Vec<CachedProviderKey> {
    let mut keys = input
        .provider
        .keys
        .iter()
        .filter(|key| key_allowed(key, input.group, input.current_minute))
        .cloned()
        .collect::<Vec<_>>();
    match input.scheduling_mode {
        ProviderSchedulingMode::FixedOrder => {
            keys.sort_by(|left, right| (left.internal_priority, &left.id).cmp(&(right.internal_priority, &right.id)));
        }
        ProviderSchedulingMode::CacheAffinity => order_keys_for_cache_affinity(&mut keys, input.affinity, input.request_id),
        ProviderSchedulingMode::LoadBalance => order_keys_for_load_balance(&mut keys, input.request_id),
    }
    keys
}
fn order_keys_for_cache_affinity(keys: &mut Vec<CachedProviderKey>, affinity: Option<&AffinitySelection>, request_id: &str) {
    if let Some(affinity) = affinity {
        promote_affinity_key(keys, &affinity.key_id);
        return;
    }
    order_keys_for_load_balance(keys, request_id);
}

fn promote_affinity_key(keys: &mut Vec<CachedProviderKey>, key_id: &str) {
    let Some(index) = keys.iter().position(|key| key.id == key_id) else {
        return;
    };
    let key = keys.remove(index);
    keys.insert(0, key);
}

fn promote_affinity_endpoint(endpoints: &mut Vec<CachedEndpoint>, affinity: Option<&AffinitySelection>) {
    let Some(affinity) = affinity else {
        return;
    };
    let Some(index) = endpoints.iter().position(|endpoint| endpoint.id == affinity.endpoint_id) else {
        return;
    };
    let endpoint = endpoints.remove(index);
    endpoints.insert(0, endpoint);
}

fn order_keys_for_load_balance(keys: &mut [CachedProviderKey], seed: &str) {
    keys.sort_by(|left, right| (stable_hash(&format!("{seed}:{}", left.id)), &left.id).cmp(&(stable_hash(&format!("{seed}:{}", right.id)), &right.id)));
}

fn provider_model(provider: &CachedProvider, model_id: &str) -> Option<CachedModelBinding> {
    provider
        .models
        .iter()
        .find(|model| model.global_model_id == model_id && model.is_active)
        .map(selected_provider_model)
}

fn endpoint_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
    endpoint.is_active && (endpoint_exact(endpoint, request) || conversion_allowed(provider, endpoint, request))
}

fn conversion_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
    if request.has_openai_responses_custom_tool_items {
        return false;
    }
    (provider.enable_format_conversion || endpoint_accepts_conversion(endpoint))
        && formats::formats_compatible(request.routing_api_format, &endpoint.api_format, request.is_stream)
        && !endpoint_exact(endpoint, request)
}

fn endpoint_exact(endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
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

fn key_allows_candidate(key: &CachedProviderKey, model_id: &str, endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
    key_allows_model(key, model_id) && key_allows_endpoint(key, endpoint) && key_supports_required_capability(key, request.required_capability)
}

fn key_allows_endpoint(key: &CachedProviderKey, endpoint: &CachedEndpoint) -> bool {
    key.api_formats.iter().any(|api_format| api_format == &endpoint.api_format)
}

fn global_model_supports_required_capability(request: CandidateRequest<'_>, snapshot: &SchedulingSnapshot, model_id: &str) -> bool {
    let Some(required) = request.required_capability else {
        return true;
    };
    snapshot
        .models
        .iter()
        .find(|model| model.id == model_id)
        .is_some_and(|model| capability_list_supports(model.supported_capabilities.as_deref(), required))
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
    candidate_supports_required_capability(key.capabilities.as_ref(), required)
}

fn capability_list_supports(capabilities: Option<&[String]>, required: &str) -> bool {
    let required = required.trim();
    capabilities.is_some_and(|items| items.iter().any(|value| value.eq_ignore_ascii_case(required)))
}

fn candidate_supports_required_capability(capabilities: Option<&serde_json::Value>, required: &str) -> bool {
    let required = required.trim();
    if required.is_empty() {
        return true;
    }
    let Some(capabilities) = capabilities else {
        return false;
    };
    if let Some(object) = capabilities.as_object() {
        return object
            .iter()
            .any(|(key, value)| key.eq_ignore_ascii_case(required) && capability_value_enabled(value));
    }
    if let Some(items) = capabilities.as_array() {
        return items
            .iter()
            .any(|value| value.as_str().is_some_and(|value| value.eq_ignore_ascii_case(required)));
    }
    false
}

fn capability_value_enabled(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(value) => *value,
        serde_json::Value::String(value) => value.eq_ignore_ascii_case("true"),
        serde_json::Value::Number(value) => value.as_i64().is_some_and(|value| value > 0),
        _ => false,
    }
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

fn stable_hash(value: &str) -> u64 {
    value
        .bytes()
        .fold(FNV_OFFSET_BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}

fn current_utc_minute() -> u16 {
    let time = OffsetDateTime::now_utc().time();
    provider_key_minute_of_day(u16::from(time.hour()), u16::from(time.minute())).expect("UTC time must have a valid minute of day")
}
