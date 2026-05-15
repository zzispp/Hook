use types::provider::ProviderSchedulingMode;

use super::helpers::{provider_b, provider_with_endpoints_and_keys, request, snapshot_with_provider, user_access};
use crate::llm_proxy::{
    cache::snapshot::SchedulingSnapshot,
    candidate::selection::matching::{MatchingCandidatePartsInput, matching_candidate_parts},
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
        affinity_key: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-a");
    assert_eq!(parts[0].endpoints.len(), 3);
    assert_eq!(parts[0].keys.len(), 2);
    assert_eq!(parts[0].endpoints[0].api_format, "openai_chat");
    assert_eq!(parts[0].endpoints[1].api_format, "gemini_chat");
    assert_eq!(parts[0].keys[0].id, "key-a-1");
}

#[test]
fn matching_candidate_parts_promotes_affinity_key_inside_route() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let group = &snapshot.groups[0];

    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request(),
        affinity_key: Some("key-a-2"),
        scheduling_mode: ProviderSchedulingMode::CacheAffinity,
        request_id: "request-1",
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].keys[0].id, "key-a-2");
    assert_eq!(parts[0].keys[1].id, "key-a-1");
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
        affinity_key: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
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
        affinity_key: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
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
        affinity_key: None,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        request_id: "request-1",
    });

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].provider.id, "provider-b");
}
