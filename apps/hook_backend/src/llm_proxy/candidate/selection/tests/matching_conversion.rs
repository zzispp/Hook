use std::collections::HashSet;

use types::provider::ProviderSchedulingMode;

use super::helpers::{provider_key, provider_with_keys, request, snapshot_with_provider};
use crate::llm_proxy::candidate::{
    CandidateRequest,
    selection::matching::{MatchingCandidatePartsInput, matching_candidate_parts},
};

#[test]
fn matching_candidate_parts_routes_gemini_request_to_openai_endpoint_key_when_converted() {
    let provider = provider_with_keys(vec![provider_key("key-openai", 10, vec!["openai_chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "gemini_chat",
            ..request()
        },
        affinity_key: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_chat");
    assert_eq!(parts[0].keys.len(), 1);
    assert_eq!(parts[0].keys[0].id, "key-openai");
}

#[test]
fn matching_candidate_parts_routes_claude_request_to_openai_endpoint_key_when_converted() {
    let provider = provider_with_keys(vec![provider_key("key-openai", 10, vec!["openai_chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "claude_chat",
            ..request()
        },
        affinity_key: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_chat");
    assert_eq!(parts[0].keys[0].id, "key-openai");
}
