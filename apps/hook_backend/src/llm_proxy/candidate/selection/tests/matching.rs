use std::collections::HashSet;

use types::provider::ProviderSchedulingMode;

use super::helpers::{
    endpoint, minute_of_day, provider_b, provider_key, provider_key_for_models, provider_key_with_time_range, provider_with_endpoints_and_keys,
    provider_with_keys, request, snapshot_with_provider, user_access,
};
use crate::llm_proxy::{
    AffinitySelection,
    cache::snapshot::SchedulingSnapshot,
    candidate::{
        CandidateRequest,
        selection::matching::{MatchingCandidatePartsInput, matching_candidate_parts, matching_candidate_parts_at},
    },
};

#[test]
fn matching_candidate_parts_builds_one_candidate_per_key() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].provider.id, "provider-a");
    assert_eq!(parts[0].endpoints.len(), 2);
    assert_eq!(parts[0].keys.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:chat");
    assert_eq!(parts[0].endpoints[1].api_format, "gemini:chat");
    assert_eq!(parts[0].keys[0].id, "key-a-1");
    assert_eq!(parts[1].keys[0].id, "key-a-2");
}

#[test]
fn matching_candidate_parts_promotes_affinity_key_inside_route() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];
    let affinity = affinity("key-a-2");

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: Some(&affinity),
        scheduling_mode: ProviderSchedulingMode::CacheAffinity,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].endpoints[0].id, "endpoint-openai");
    assert_eq!(parts[0].keys[0].id, "key-a-2");
    assert_eq!(parts[1].keys[0].id, "key-a-1");
}

fn affinity(key_id: &str) -> AffinitySelection {
    AffinitySelection {
        provider_id: "provider-a".into(),
        endpoint_id: "endpoint-openai".into(),
        key_id: key_id.into(),
    }
}

#[test]
fn matching_candidate_parts_keeps_all_provider_routes_without_silent_budget() {
    let snapshot = SchedulingSnapshot {
        providers: vec![provider_with_endpoints_and_keys(), provider_b()],
        ..snapshot_with_provider(provider_with_endpoints_and_keys())
    };
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].provider.id, "provider-a");
    assert_eq!(parts[2].provider.id, "provider-b");
}

#[test]
fn matching_candidate_parts_uses_key_level_effective_provider_model() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].effective_upstream_model_name, "mapped-upstream-model");
    assert_eq!(parts[0].effective_reasoning_effort.as_deref(), Some("high"));
}

#[test]
fn matching_candidate_parts_filters_by_user_provider_access() {
    let snapshot = SchedulingSnapshot {
        providers: vec![provider_with_endpoints_and_keys(), provider_b()],
        ..snapshot_with_provider(provider_with_endpoints_and_keys())
    };
    let group = &snapshot.groups[0];
    let user_access = user_access("user-a", "alice", vec!["provider-b".into()]);

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: Some(&user_access),
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-b");
}

#[test]
fn matching_candidate_parts_filters_cooled_providers() {
    let snapshot = SchedulingSnapshot {
        providers: vec![provider_with_endpoints_and_keys(), provider_b()],
        ..snapshot_with_provider(provider_with_endpoints_and_keys())
    };
    let group = &snapshot.groups[0];
    let cooled_provider_ids = HashSet::from(["provider-a".to_owned()]);

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &cooled_provider_ids,
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-b");
}

#[test]
fn matching_candidate_parts_filters_all_keys_for_cooled_provider_in_key_mode() {
    let snapshot = SchedulingSnapshot {
        provider_priority_mode: types::provider::ProviderPriorityMode::Key,
        providers: vec![provider_with_endpoints_and_keys(), provider_b()],
        ..snapshot_with_provider(provider_with_endpoints_and_keys())
    };
    let group = &snapshot.groups[0];
    let cooled_provider_ids = HashSet::from(["provider-a".to_owned()]);

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &cooled_provider_ids,
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-b");
    assert!(parts.iter().all(|part| part.keys[0].provider_id != "provider-a"));
}

#[test]
fn matching_candidate_parts_does_not_route_chat_request_to_non_chat_endpoint() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts[0].endpoints.iter().all(|endpoint| endpoint.api_format != "openai_image"));
}

#[test]
fn matching_candidate_parts_routes_explicit_image_intent_to_image_endpoint() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai_image",
            model_name: "gpt-test",
            is_stream: false,
            has_openai_responses_custom_tool_items: false,
            required_capability: Some("image_generation"),
            features: types::provider::RoutingRequestFeatures::unknown("openai:cli", false, Some("image_generation")),
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 2);
    assert!(parts.iter().all(|part| part.client_api_format == "openai:cli"));
    assert!(parts.iter().all(|part| part.routing_api_format == "openai_image"));
    assert!(parts.iter().all(|part| part.endpoints[0].api_format == "openai_image"));
}

#[test]
fn matching_candidate_parts_does_not_route_stream_responses_to_compact_endpoint() {
    let provider = provider_with_responses_and_compact_endpoints(vec![
        provider_key("key-responses", 10, vec!["openai:cli"]),
        provider_key("key-compact", 20, vec!["openai:compact"]),
    ]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai:cli",
            model_name: "gpt-test",
            is_stream: true,
            has_openai_responses_custom_tool_items: false,
            required_capability: None,
            features: types::provider::RoutingRequestFeatures::unknown("openai:cli", true, None),
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
fn matching_candidate_parts_does_not_treat_responses_compact_as_exact_route() {
    let provider = provider_with_responses_and_compact_endpoints(vec![provider_key("key-compact", 10, vec!["openai:compact"])]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai:cli",
            model_name: "gpt-test",
            is_stream: false,
            has_openai_responses_custom_tool_items: false,
            required_capability: None,
            features: types::provider::RoutingRequestFeatures::unknown("openai:cli", false, None),
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts.is_empty());
}

#[test]
fn matching_candidate_parts_routes_responses_request_to_chat_endpoint_through_conversion() {
    let provider = provider_with_responses_and_chat_endpoints(vec![provider_key("key-chat", 10, vec!["openai:chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai:cli",
            model_name: "gpt-test",
            is_stream: true,
            has_openai_responses_custom_tool_items: false,
            required_capability: None,
            features: types::provider::RoutingRequestFeatures::unknown("openai:cli", true, None),
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai:chat");
}

fn provider_with_responses_and_compact_endpoints(
    keys: Vec<crate::llm_proxy::cache::snapshot::CachedProviderKey>,
) -> crate::llm_proxy::cache::snapshot::CachedProvider {
    crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-responses", "openai:cli"), endpoint("endpoint-compact", "openai:compact")],
        keys,
        ..provider_with_endpoints_and_keys()
    }
}

fn provider_with_responses_and_chat_endpoints(
    keys: Vec<crate::llm_proxy::cache::snapshot::CachedProviderKey>,
) -> crate::llm_proxy::cache::snapshot::CachedProvider {
    crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-responses", "openai:cli"), endpoint("endpoint-chat", "openai:chat")],
        keys,
        ..provider_with_endpoints_and_keys()
    }
}

#[test]
fn matching_candidate_parts_routes_non_chat_request_only_to_matching_data_format() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai_image",
            routing_api_format: "openai_image",
            model_name: "gpt-test",
            is_stream: false,
            has_openai_responses_custom_tool_items: false,
            required_capability: Some("image_generation"),
            features: types::provider::RoutingRequestFeatures::unknown("openai_image", false, Some("image_generation")),
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 2);
    assert!(parts.iter().all(|part| part.endpoints.len() == 1));
    assert!(parts.iter().all(|part| part.endpoints[0].api_format == "openai_image"));
}

#[test]
fn matching_candidate_parts_routes_image_edit_only_to_exact_edit_endpoint() {
    let provider = provider_with_keys(vec![provider_key("key-image-edit", 10, vec!["openai_image_edit"])]);
    let snapshot = snapshot_with_provider(crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-image", "openai_image"), endpoint("endpoint-image-edit", "openai_image_edit")],
        ..provider
    });
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai_image_edit",
            routing_api_format: "openai_image_edit",
            model_name: "gpt-test",
            is_stream: false,
            has_openai_responses_custom_tool_items: false,
            required_capability: Some("image_generation"),
            features: types::provider::RoutingRequestFeatures::unknown("openai_image_edit", false, Some("image_generation")),
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_image_edit");
}

#[test]
fn matching_candidate_parts_requires_global_model_image_generation_capability() {
    let mut snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    snapshot.models[0].supported_capabilities = None;
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai_image",
            model_name: "gpt-test",
            is_stream: false,
            has_openai_responses_custom_tool_items: false,
            required_capability: Some("image_generation"),
            features: types::provider::RoutingRequestFeatures::unknown("openai:cli", false, Some("image_generation")),
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 0);
}

#[test]
fn matching_candidate_parts_requires_provider_key_image_generation_capability() {
    let mut capable_key = provider_key("key-capable", 10, vec!["openai_image"]);
    capable_key.capabilities = Some(serde_json::json!({ "image_generation": true }));
    let mut missing_capability_key = provider_key("key-missing-capability", 20, vec!["openai_image"]);
    missing_capability_key.capabilities = None;
    let provider = provider_with_keys(vec![missing_capability_key, capable_key]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai:cli",
            routing_api_format: "openai_image",
            model_name: "gpt-test",
            is_stream: false,
            has_openai_responses_custom_tool_items: false,
            required_capability: Some("image_generation"),
            features: types::provider::RoutingRequestFeatures::unknown("openai:cli", false, Some("image_generation")),
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].keys[0].id, "key-capable");
    assert_eq!(parts[0].endpoints[0].api_format, "openai_image");
}

#[test]
fn matching_candidate_parts_requires_key_to_support_endpoint_format() {
    let provider = provider_with_keys(vec![
        provider_key("key-openai", 10, vec!["openai:chat"]),
        provider_key("key-gemini", 20, vec!["gemini:chat"]),
    ]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 2);
    assert!(parts.iter().all(|part| part.keys.len() == 1));
    assert!(
        parts
            .iter()
            .any(|part| part.keys[0].id == "key-openai" && part.endpoints.iter().any(|endpoint| endpoint.api_format == "openai:chat"))
    );
    assert!(
        parts
            .iter()
            .any(|part| part.keys[0].id == "key-gemini" && part.endpoints.iter().any(|endpoint| endpoint.api_format == "gemini:chat"))
    );
}

#[test]
fn matching_candidate_parts_excludes_key_with_empty_api_formats() {
    let provider = provider_with_keys(vec![provider_key("key-empty", 10, Vec::new())]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts.is_empty());
}

#[test]
fn matching_candidate_parts_excludes_key_that_does_not_allow_requested_model() {
    let provider = provider_with_keys(vec![provider_key_for_models("key-model-b", 10, vec!["openai:chat"], vec!["model-b"])]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts.is_empty());
}

#[test]
fn matching_candidate_parts_skips_key_outside_enabled_time_range() {
    let provider = provider_with_keys(vec![
        provider_key_with_time_range("key-current", 10, minute_of_day(8, 0), minute_of_day(18, 0)),
        provider_key_with_time_range("key-expired", 20, minute_of_day(18, 0), minute_of_day(20, 0)),
    ]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts_at(
        MatchingCandidatePartsInput {
            snapshot: &snapshot,
            group,
            user_access: None,
            model_id: "model-a",
            request: request(),
            affinity: None,
            scheduling_mode: ProviderSchedulingMode::FixedOrder,
            request_id: "request-1",
            cooled_provider_ids: &HashSet::new(),
        },
        minute_of_day(10, 0),
    );

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].keys.len(), 1);
    assert_eq!(parts[0].keys[0].id, "key-current");
}

#[test]
fn matching_candidate_parts_treats_empty_key_allowed_models_as_all_models() {
    let provider = provider_with_keys(vec![provider_key("key-all-models", 10, vec!["openai:chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].keys[0].id, "key-all-models");
}
