use provider::application::SecretCipher;
use types::api_token::ApiToken;

use super::{CandidateParts, DEFAULT_MAX_RETRIES, GlobalModelRef};
use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    cache::snapshot::CachedBillingGroup,
    candidate::{CandidateRequest, CandidateTrace, ProxyCandidate, url},
};

pub(super) async fn proxy_candidates(
    state: &LlmProxyState,
    token: &ApiToken,
    request: CandidateRequest<'_>,
    global_model: &GlobalModelRef,
    group: &CachedBillingGroup,
    parts: &[CandidateParts],
) -> Result<Vec<ProxyCandidate>, LlmProxyError> {
    let mut candidates = Vec::with_capacity(parts.len());
    for (index, part) in parts.iter().enumerate() {
        candidates.push(proxy_candidate(state, token, request, global_model, group, part, index as i32).await?);
    }
    Ok(candidates)
}

async fn proxy_candidate(
    state: &LlmProxyState,
    token: &ApiToken,
    request: CandidateRequest<'_>,
    global_model: &GlobalModelRef,
    group: &CachedBillingGroup,
    parts: &CandidateParts,
    index: i32,
) -> Result<ProxyCandidate, LlmProxyError> {
    let api_key = state.cipher.decrypt_provider_key(&parts.key.encrypted_api_key)?;
    Ok(ProxyCandidate {
        trace: candidate_trace(token, request, parts, index),
        api_key,
        base_url: parts.endpoint.base_url.clone(),
        custom_path: parts.endpoint.custom_path.clone(),
        upstream_url: url::upstream_url(parts, request.is_stream),
        provider_model_name: parts.model.provider_model_name.clone(),
        price_per_request: parts.model.price_per_request.or(global_model.default_price_per_request),
        tiered_pricing: parts
            .model
            .tiered_pricing
            .clone()
            .unwrap_or_else(|| global_model.default_tiered_pricing.clone()),
        billing_multiplier: group.billing_multiplier,
        max_retries: max_retries(parts),
        request_timeout_seconds: parts.provider.request_timeout_seconds,
        stream_first_byte_timeout_seconds: parts.provider.stream_first_byte_timeout_seconds,
        cache_ttl_minutes: parts.key.cache_ttl_minutes,
    })
}

fn candidate_trace(token: &ApiToken, request: CandidateRequest<'_>, parts: &CandidateParts, index: i32) -> CandidateTrace {
    CandidateTrace {
        token_id: Some(token.id.clone()),
        group_code: Some(token.group_code.clone()),
        global_model_id: parts.model.global_model_id.clone(),
        provider_id: parts.provider.id.clone(),
        endpoint_id: parts.endpoint.id.clone(),
        key_id: parts.key.id.clone(),
        client_api_format: request.api_format.to_owned(),
        provider_api_format: parts.endpoint.api_format.clone(),
        needs_conversion: parts.needs_conversion,
        is_stream: request.is_stream,
        candidate_index: index,
    }
}

fn max_retries(parts: &CandidateParts) -> i32 {
    parts.endpoint.max_retries.or(parts.provider.max_retries).unwrap_or(DEFAULT_MAX_RETRIES).max(0)
}
