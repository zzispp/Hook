use super::{
    Candidate, EndpointSnapshot, KeySnapshot, ModelAccessPolicy, ModelBindingSnapshot, ProviderSnapshot, SchedulerError, SchedulerInput, SchedulingMode,
};

const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

pub struct CandidateBuilder;

impl CandidateBuilder {
    pub fn build(input: &SchedulerInput) -> Result<Vec<Candidate>, SchedulerError> {
        validate_scope(input)?;
        let mut candidates = collect_candidates(input);
        if candidates.is_empty() {
            return Err(SchedulerError::NoModelCandidate {
                model: input.requested_model_id.clone(),
            });
        }
        sort_candidates(&mut candidates, input);
        Ok(candidates)
    }

    pub fn order(candidates: &mut Vec<Candidate>, input: &SchedulerInput) {
        sort_candidates(candidates, input);
    }
}

fn validate_scope(input: &SchedulerInput) -> Result<(), SchedulerError> {
    if !input.group_is_active {
        return Err(SchedulerError::InactiveGroup(input.group_code.clone()));
    }
    if !policy_allows(&input.token_model_policy, &input.requested_model_id) {
        return Err(SchedulerError::TokenModelDenied {
            model: input.requested_model_id.clone(),
        });
    }
    if !ids_allow(&input.group_allowed_model_ids, &input.requested_model_id) {
        return Err(SchedulerError::GroupModelDenied {
            group_code: input.group_code.clone(),
            model: input.requested_model_id.clone(),
        });
    }
    if !ids_allow(&input.user_allowed_model_ids, &input.requested_model_id) {
        return Err(SchedulerError::UserModelDenied {
            model: input.requested_model_id.clone(),
        });
    }
    Ok(())
}

fn collect_candidates(input: &SchedulerInput) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for provider in input.providers.iter().filter(|provider| provider_allowed(provider, input)) {
        append_provider_candidates(provider, input, &mut candidates);
    }
    candidates
}

fn append_provider_candidates(provider: &ProviderSnapshot, input: &SchedulerInput, candidates: &mut Vec<Candidate>) {
    let Some(model) = matching_model(provider, input) else {
        return;
    };
    let mut exact = Vec::new();
    let mut convertible = Vec::new();
    for endpoint in provider.endpoints.iter().filter(|endpoint| endpoint.is_active) {
        if !input_compatible(endpoint, input, provider) {
            continue;
        }
        let target = if endpoint.api_format == input.client_format {
            &mut exact
        } else {
            &mut convertible
        };
        append_endpoint_candidates(
            EndpointBuildContext {
                provider,
                endpoint,
                model,
                input,
            },
            target,
        );
    }
    candidates.extend(exact);
    candidates.extend(convertible);
}

struct EndpointBuildContext<'a> {
    provider: &'a ProviderSnapshot,
    endpoint: &'a EndpointSnapshot,
    model: &'a ModelBindingSnapshot,
    input: &'a SchedulerInput,
}

fn append_endpoint_candidates(context: EndpointBuildContext<'_>, output: &mut Vec<Candidate>) {
    let provider = context.provider;
    let endpoint = context.endpoint;
    let model = context.model;
    for key in provider.keys.iter().filter(|key| key_allowed(key)) {
        output.push(Candidate {
            provider_id: provider.id.clone(),
            provider_name: provider.name.clone(),
            endpoint_id: endpoint.id.clone(),
            key_id: key.id.clone(),
            global_model_id: model.global_model_id.clone(),
            provider_model_name: model.provider_model_name.clone(),
            provider_api_format: endpoint.api_format,
            needs_conversion: endpoint.api_format != context.input.client_format,
            is_cached: false,
            provider_priority: provider.priority,
            key_priority: key.internal_priority,
        });
    }
}

fn sort_candidates(candidates: &mut Vec<Candidate>, input: &SchedulerInput) {
    set_conversion_flags(candidates, input);
    candidates.sort_by(stable_priority);
    demote_conversion(candidates, input);
    match input.scheduling_mode {
        SchedulingMode::FixedOrder => {}
        SchedulingMode::CacheAffinity => apply_cache_affinity(candidates, input.affinity_key.as_deref()),
        SchedulingMode::LoadBalance => apply_load_balance(candidates, input),
    }
}

fn provider_allowed(provider: &ProviderSnapshot, input: &SchedulerInput) -> bool {
    provider.is_active && ids_allow(&input.group_allowed_provider_ids, &provider.id) && ids_allow(&input.user_allowed_provider_ids, &provider.id)
}

fn matching_model<'a>(provider: &'a ProviderSnapshot, input: &SchedulerInput) -> Option<&'a ModelBindingSnapshot> {
    provider.models.iter().find(|model| model.global_model_id == input.requested_model_id)
}

fn input_compatible(endpoint: &EndpointSnapshot, input: &SchedulerInput, provider: &ProviderSnapshot) -> bool {
    if endpoint.api_format == input.client_format {
        return true;
    }
    if input.is_stream && !endpoint.supports_stream_conversion {
        return false;
    }
    input.global_format_conversion_enabled || provider.enable_format_conversion || endpoint.accepts_format_conversion
}

fn key_allowed(key: &KeySnapshot) -> bool {
    key.is_active
}

fn policy_allows(policy: &ModelAccessPolicy, model_id: &str) -> bool {
    match policy {
        ModelAccessPolicy::All => true,
        ModelAccessPolicy::Limited(ids) => ids.iter().any(|id| id == model_id),
    }
}

fn ids_allow(ids: &[String], id: &str) -> bool {
    ids.is_empty() || ids.iter().any(|item| item == id)
}

fn set_conversion_flags(candidates: &mut [Candidate], input: &SchedulerInput) {
    for candidate in candidates {
        candidate.needs_conversion = candidate.provider_api_format != input.client_format;
    }
}

fn demote_conversion(candidates: &mut Vec<Candidate>, input: &SchedulerInput) {
    if input.global_keep_priority_on_conversion {
        return;
    }
    let mut exact = Vec::new();
    let mut converted = Vec::new();
    for candidate in candidates.drain(..) {
        if candidate.needs_conversion && should_demote(&candidate, input) {
            converted.push(candidate);
        } else {
            exact.push(candidate);
        }
    }
    exact.extend(converted);
    *candidates = exact;
}

fn should_demote(candidate: &Candidate, input: &SchedulerInput) -> bool {
    !input
        .providers
        .iter()
        .find(|provider| provider.id == candidate.provider_id)
        .is_some_and(|provider| provider.keep_priority_on_conversion)
}

fn apply_cache_affinity(candidates: &mut Vec<Candidate>, affinity_key: Option<&str>) {
    let Some(key_id) = affinity_key else {
        return;
    };
    let Some(index) = candidates.iter().position(|candidate| candidate.key_id == key_id) else {
        return;
    };
    let mut candidate = candidates.remove(index);
    candidate.is_cached = true;
    candidates.insert(0, candidate);
}

fn apply_load_balance(candidates: &mut [Candidate], input: &SchedulerInput) {
    candidates.sort_by(|left, right| load_balance_key(left, input).cmp(&load_balance_key(right, input)));
}

fn stable_priority(left: &Candidate, right: &Candidate) -> std::cmp::Ordering {
    (left.provider_priority, left.key_priority, &left.provider_id, &left.endpoint_id, &left.key_id).cmp(&(
        right.provider_priority,
        right.key_priority,
        &right.provider_id,
        &right.endpoint_id,
        &right.key_id,
    ))
}

fn load_balance_key<'a>(candidate: &'a Candidate, input: &SchedulerInput) -> (i32, i32, i32, u64, &'a str) {
    (
        conversion_rank(candidate, input),
        candidate.provider_priority,
        candidate.key_priority,
        stable_hash(&format!("{}:{}", input.load_balance_seed.as_deref().unwrap_or_default(), candidate.key_id)),
        candidate.key_id.as_str(),
    )
}

fn conversion_rank(candidate: &Candidate, input: &SchedulerInput) -> i32 {
    if candidate.needs_conversion && should_demote(candidate, input) {
        return 1;
    }
    0
}

fn stable_hash(value: &str) -> u64 {
    value
        .bytes()
        .fold(FNV_OFFSET_BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}
