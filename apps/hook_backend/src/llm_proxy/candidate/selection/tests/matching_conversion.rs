use std::collections::HashSet;

use types::provider::ProviderSchedulingMode;

use super::helpers::{endpoint, provider_key, provider_with_endpoints_and_keys, provider_with_keys, request, snapshot_with_provider};
use crate::llm_proxy::candidate::{
    CandidateRequest,
    selection::matching::{MatchingCandidatePartsInput, matching_candidate_parts},
};

#[test]
fn matching_candidate_parts_routes_gemini_request_to_openai_endpoint_key_when_converted() {
    let provider = provider_with_keys(vec![provider_key("key-openai", 10, vec!["openai:chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "gemini:chat",
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:chat");
    assert_eq!(parts[0].keys.len(), 1);
    assert_eq!(parts[0].keys[0].id, "key-openai");
}

#[test]
fn matching_candidate_parts_routes_claude_request_to_openai_endpoint_key_when_converted() {
    let provider = provider_with_keys(vec![provider_key("key-openai", 10, vec!["openai:chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "claude:chat",
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:chat");
    assert_eq!(parts[0].keys[0].id, "key-openai");
}

#[test]
fn matching_candidate_parts_routes_claude_request_to_openai_cli_endpoint_when_converted() {
    let provider = crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-chat", "openai:chat"), endpoint("endpoint-cli", "openai:cli")],
        keys: vec![
            provider_key("key-chat", 10, vec!["openai:chat"]),
            provider_key("key-cli", 20, vec!["openai:cli"]),
        ],
        ..provider_with_endpoints_and_keys()
    };
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "claude:chat",
            is_stream: true,
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 2);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:cli");
    assert_eq!(parts[0].endpoints[1].api_format, "openai:chat");
}
