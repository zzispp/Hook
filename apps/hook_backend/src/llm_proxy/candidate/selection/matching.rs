use types::provider::ProviderSchedulingMode;

use super::CandidateParts;
use crate::llm_proxy::{
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
    pub(super) affinity_key: Option<&'a str>,
    pub(super) scheduling_mode: ProviderSchedulingMode,
    pub(super) request_id: &'a str,
}

pub(super) fn matching_candidate_parts(input: MatchingCandidatePartsInput<'_>) -> Vec<CandidateParts> {
    let mut candidates = Vec::new();
    for provider in input
        .snapshot
        .providers
        .iter()
        .filter(|provider| provider_allowed(input.group, input.user_access, provider))
    {
        append_provider_candidate(
            AppendProviderCandidateInput {
                provider,
                model_id: input.model_id,
                request: input.request,
                affinity_key: input.affinity_key,
                scheduling_mode: input.scheduling_mode,
                request_id: input.request_id,
            },
            &mut candidates,
        );
    }
    candidates
}

struct AppendProviderCandidateInput<'a> {
    provider: &'a CachedProvider,
    model_id: &'a str,
    request: CandidateRequest<'a>,
    affinity_key: Option<&'a str>,
    scheduling_mode: ProviderSchedulingMode,
    request_id: &'a str,
}

fn append_provider_candidate(input: AppendProviderCandidateInput<'_>, output: &mut Vec<CandidateParts>) {
    let Some(model) = provider_model(input.provider, input.model_id, input.affinity_key) else {
        return;
    };
    let endpoints = ordered_endpoints(input.provider, input.request);
    let keys = ordered_keys(OrderedKeysInput {
        provider: input.provider,
        affinity_key: input.affinity_key,
        scheduling_mode: input.scheduling_mode,
        request_id: input.request_id,
    });
    if endpoints.is_empty() || keys.is_empty() {
        return;
    }
    output.push(CandidateParts {
        provider: input.provider.clone(),
        endpoints,
        keys,
        model,
        client_api_format: input.request.api_format.to_owned(),
    });
}

struct OrderedKeysInput<'a> {
    provider: &'a CachedProvider,
    affinity_key: Option<&'a str>,
    scheduling_mode: ProviderSchedulingMode,
    request_id: &'a str,
}

fn ordered_endpoints(provider: &CachedProvider, request: CandidateRequest<'_>) -> Vec<CachedEndpoint> {
    let (mut exact, converted): (Vec<_>, Vec<_>) = provider
        .endpoints
        .iter()
        .filter(|endpoint| endpoint_allowed(provider, endpoint, request))
        .cloned()
        .partition(|endpoint| endpoint.api_format == request.api_format);
    exact.extend(converted);
    exact
}

fn ordered_keys(input: OrderedKeysInput<'_>) -> Vec<CachedProviderKey> {
    let mut keys = input.provider.keys.iter().filter(|key| key_allowed(key)).cloned().collect::<Vec<_>>();
    keys.sort_by(|left, right| (left.internal_priority, &left.id).cmp(&(right.internal_priority, &right.id)));
    match input.scheduling_mode {
        ProviderSchedulingMode::CacheAffinity => order_keys_for_cache_affinity(&mut keys, input.affinity_key, input.request_id),
        ProviderSchedulingMode::LoadBalance => order_keys_for_load_balance(&mut keys, input.request_id),
        ProviderSchedulingMode::FixedOrder => {}
    }
    keys
}
fn order_keys_for_cache_affinity(keys: &mut Vec<CachedProviderKey>, affinity_key: Option<&str>, request_id: &str) {
    if affinity_key.is_some() {
        promote_affinity_key(keys, affinity_key);
        return;
    }
    order_keys_for_load_balance(keys, request_id);
}

fn promote_affinity_key(keys: &mut Vec<CachedProviderKey>, affinity_key: Option<&str>) {
    let Some(key_id) = affinity_key else {
        return;
    };
    let Some(index) = keys.iter().position(|key| key.id == key_id) else {
        return;
    };
    let key = keys.remove(index);
    keys.insert(0, key);
}

fn order_keys_for_load_balance(keys: &mut [CachedProviderKey], seed: &str) {
    keys.sort_by(|left, right| {
        (left.internal_priority, stable_hash(&format!("{seed}:{}", left.id)), &left.id).cmp(&(
            right.internal_priority,
            stable_hash(&format!("{seed}:{}", right.id)),
            &right.id,
        ))
    });
}

fn provider_model(provider: &CachedProvider, model_id: &str, affinity_key: Option<&str>) -> Option<CachedModelBinding> {
    provider
        .models
        .iter()
        .find(|model| model.global_model_id == model_id && model.is_active)
        .map(|model| selected_provider_model(model, affinity_key))
}

fn endpoint_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
    endpoint.is_active && (endpoint.api_format == request.api_format || conversion_allowed(provider, endpoint, request))
}

fn conversion_allowed(provider: &CachedProvider, endpoint: &CachedEndpoint, request: CandidateRequest<'_>) -> bool {
    (provider.enable_format_conversion || endpoint_accepts_conversion(endpoint))
        && formats::formats_compatible(request.api_format, &endpoint.api_format, request.is_stream)
}

fn endpoint_accepts_conversion(endpoint: &CachedEndpoint) -> bool {
    endpoint
        .format_acceptance_config
        .as_ref()
        .and_then(|value| value.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn key_allowed(key: &CachedProviderKey) -> bool {
    key.is_active
}

fn selected_provider_model(model: &CachedModelBinding, _affinity_key: Option<&str>) -> CachedModelBinding {
    let mut selected = model.clone();
    selected.provider_model_name = selected_provider_model_name(model);
    selected
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
