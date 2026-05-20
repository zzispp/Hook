use std::ops::RangeInclusive;

use super::{LlmProxyError, LlmProxyState, failure_classification::FailureDecision};
use crate::llm_proxy::{InvalidateAffinityInput, SetAffinityInput, candidate::ProxyCandidate};

pub(super) fn attempt_range(candidate: &ProxyCandidate) -> RangeInclusive<i32> {
    0..=effective_max_retries(candidate)
}

pub(super) async fn remember(state: &LlmProxyState, candidate: &ProxyCandidate, ttl_minutes: i64) -> Result<(), LlmProxyError> {
    let Some(input) = set_affinity_input(candidate, ttl_minutes) else {
        return Ok(());
    };
    state.remember_affinity(input).await
}

pub(super) async fn invalidate_matching(state: &LlmProxyState, candidate: &ProxyCandidate) -> Result<(), LlmProxyError> {
    let Some(input) = invalidate_affinity_input(candidate) else {
        return Ok(());
    };
    state.invalidate_affinity(input).await
}

pub(super) async fn invalidate_retryable(state: &LlmProxyState, candidate: &ProxyCandidate, decision: FailureDecision) -> Result<(), LlmProxyError> {
    if !matches!(decision, FailureDecision::RetryOrNextCandidate) {
        return Ok(());
    }
    invalidate_matching(state, candidate).await
}

fn effective_max_retries(candidate: &ProxyCandidate) -> i32 {
    candidate.max_attempt_index()
}

fn set_affinity_input(candidate: &ProxyCandidate, ttl_minutes: i64) -> Option<SetAffinityInput<'_>> {
    Some(SetAffinityInput {
        token_id: candidate.trace.token_id.as_deref()?,
        model_id: &candidate.trace.global_model_id,
        api_format: &candidate.trace.client_api_format,
        provider_id: &candidate.trace.provider_id,
        endpoint_id: &candidate.trace.endpoint_id,
        key_id: &candidate.trace.key_id,
        ttl_minutes,
    })
}

fn invalidate_affinity_input(candidate: &ProxyCandidate) -> Option<InvalidateAffinityInput<'_>> {
    Some(InvalidateAffinityInput {
        token_id: candidate.trace.token_id.as_deref()?,
        model_id: &candidate.trace.global_model_id,
        api_format: &candidate.trace.client_api_format,
        provider_id: &candidate.trace.provider_id,
        endpoint_id: &candidate.trace.endpoint_id,
        key_id: &candidate.trace.key_id,
    })
}
