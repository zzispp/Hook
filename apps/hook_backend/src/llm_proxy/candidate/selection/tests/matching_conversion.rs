use std::collections::HashSet;

use types::provider::ProviderSchedulingMode;

use super::helpers::{endpoint, provider_key, provider_with_endpoints_and_keys, provider_with_keys, request, snapshot_with_provider};
use crate::llm_proxy::candidate::{
    CandidateRequest,
    selection::matching::{MatchingCandidatePartsInput, matching_candidate_parts},
};

#[test]
fn matching_candidate_parts_excludes_openai_chat_for_gemini_request_conversion() {
    let provider = provider_with_keys(vec![provider_key("key-openai", 10, vec!["openai:chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "gemini:chat",
            routing_api_format: "gemini:chat",
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts.is_empty());
}

#[test]
fn matching_candidate_parts_routes_gemini_request_to_openai_cli_when_converted() {
    let provider = crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-openai", "openai:chat"), endpoint("endpoint-cli", "openai:cli")],
        keys: vec![
            provider_key("key-openai", 10, vec!["openai:chat"]),
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
            api_format: "gemini:chat",
            routing_api_format: "gemini:chat",
            is_stream: true,
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:cli");
    assert!(parts[0].keys.iter().any(|key| key.id == "key-cli"));
}

#[test]
fn matching_candidate_parts_excludes_openai_chat_for_claude_request_conversion() {
    let provider = provider_with_keys(vec![provider_key("key-openai", 10, vec!["openai:chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "claude:chat",
            routing_api_format: "claude:chat",
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts.is_empty());
}

#[test]
fn matching_candidate_parts_routes_claude_request_to_openai_cli_when_converted() {
    let provider = crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-openai", "openai:chat"), endpoint("endpoint-cli", "openai:cli")],
        keys: vec![
            provider_key("key-openai", 10, vec!["openai:chat"]),
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
            routing_api_format: "claude:chat",
            is_stream: true,
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:cli");
}

#[test]
fn matching_candidate_parts_routes_responses_image_generation_call_requests_to_conversion_endpoint() {
    let provider = crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-chat", "openai:chat"), endpoint("endpoint-cli", "openai:cli")],
        keys: vec![provider_key("key-both", 10, vec!["openai:cli", "openai:chat"])],
        ..provider_with_endpoints_and_keys()
    };
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai:cli",
            is_stream: true,
            has_openai_responses_custom_tool_items: false,
            required_capability: None,
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 2);
    assert!(parts[0].endpoints.iter().any(|endpoint| endpoint.api_format == "openai:chat"));
    assert!(parts[0].endpoints.iter().any(|endpoint| endpoint.api_format == "openai:cli"));
}

#[test]
fn matching_candidate_parts_keeps_responses_custom_tool_requests_on_responses_endpoint() {
    let provider = crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-chat", "openai:chat"), endpoint("endpoint-cli", "openai:cli")],
        keys: vec![provider_key("key-both", 10, vec!["openai:cli", "openai:chat"])],
        ..provider_with_endpoints_and_keys()
    };
    let snapshot = snapshot_with_provider(provider);
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai:cli",
            is_stream: true,
            has_openai_responses_custom_tool_items: true,
            required_capability: None,
            ..request()
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:cli");
}
