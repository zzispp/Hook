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
fn matching_candidate_parts_compacts_endpoint_key_product_into_provider_route() {
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

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-a");
    assert_eq!(parts[0].endpoints.len(), 2);
    assert_eq!(parts[0].keys.len(), 2);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_chat");
    assert_eq!(parts[0].endpoints[1].api_format, "gemini_chat");
    assert_eq!(parts[0].keys[0].id, "key-a-1");
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

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints[0].id, "endpoint-openai");
    assert_eq!(parts[0].keys[0].id, "key-a-2");
    assert_eq!(parts[0].keys[1].id, "key-a-1");
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

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].provider.id, "provider-a");
    assert_eq!(parts[1].provider.id, "provider-b");
}

#[test]
fn matching_candidate_parts_prefers_highest_priority_mapped_provider_model_name() {
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

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].model.provider_model_name, "mapped-upstream-model");
    assert_eq!(
        parts[0]
            .model
            .provider_model_mapping
            .as_ref()
            .and_then(|mapping| mapping.reasoning_effort.as_deref()),
        Some("high")
    );
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
fn matching_candidate_parts_does_not_route_stream_responses_to_compact_endpoint() {
    let provider = provider_with_responses_and_compact_endpoints(vec![
        provider_key("key-responses", 10, vec!["openai_cli"]),
        provider_key("key-compact", 20, vec!["openai_compact"]),
    ]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai_cli",
            model_name: "gpt-test",
            is_stream: true,
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_cli");
}

#[test]
fn matching_candidate_parts_does_not_treat_responses_compact_as_exact_route() {
    let provider = provider_with_responses_and_compact_endpoints(vec![provider_key("key-compact", 10, vec!["openai_compact"])]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai_cli",
            model_name: "gpt-test",
            is_stream: false,
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts.is_empty());
}

#[test]
fn matching_candidate_parts_does_not_route_responses_request_to_chat_endpoint() {
    let provider = provider_with_responses_and_chat_endpoints(vec![provider_key("key-chat", 10, vec!["openai_chat"])]);
    let snapshot = snapshot_with_provider(provider);
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: CandidateRequest {
            api_format: "openai_cli",
            model_name: "gpt-test",
            is_stream: true,
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert!(parts.is_empty());
}

fn provider_with_responses_and_compact_endpoints(
    keys: Vec<crate::llm_proxy::cache::snapshot::CachedProviderKey>,
) -> crate::llm_proxy::cache::snapshot::CachedProvider {
    crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-responses", "openai_cli"), endpoint("endpoint-compact", "openai_compact")],
        keys,
        ..provider_with_endpoints_and_keys()
    }
}

fn provider_with_responses_and_chat_endpoints(
    keys: Vec<crate::llm_proxy::cache::snapshot::CachedProviderKey>,
) -> crate::llm_proxy::cache::snapshot::CachedProvider {
    crate::llm_proxy::cache::snapshot::CachedProvider {
        endpoints: vec![endpoint("endpoint-responses", "openai_cli"), endpoint("endpoint-chat", "openai_chat")],
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
            model_name: "gpt-test",
            is_stream: false,
        },
        affinity: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
        cooled_provider_ids: &HashSet::new(),
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 1);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_image");
}

#[test]
fn matching_candidate_parts_requires_key_to_support_endpoint_format() {
    let provider = provider_with_keys(vec![
        provider_key("key-openai", 10, vec!["openai_chat"]),
        provider_key("key-gemini", 20, vec!["gemini_chat"]),
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

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].endpoints.len(), 2);
    assert_eq!(parts[0].keys.len(), 2);
    assert!(parts[0].endpoints.iter().any(|endpoint| endpoint.api_format == "openai_chat"));
    assert!(parts[0].endpoints.iter().any(|endpoint| endpoint.api_format == "gemini_chat"));
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
    let provider = provider_with_keys(vec![provider_key_for_models("key-model-b", 10, vec!["openai_chat"], vec!["model-b"])]);
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
    let provider = provider_with_keys(vec![provider_key("key-all-models", 10, vec!["openai_chat"])]);
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
