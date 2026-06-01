use types::{api_token::ApiTokenType, provider::ProviderPriorityMode};

use super::helpers::{api_token, provider_with_endpoints_and_keys, request, snapshot_with_provider};
use crate::llm_proxy::candidate::selection::{
    matching::{MatchingCandidatePartsInput, matching_candidate_parts},
    scheduler::{OrderCandidatePartsInput, order_candidate_parts},
};

#[test]
fn order_candidate_parts_provider_mode_uses_internal_priority() {
    let snapshot = snapshot_with_provider(provider_with_mismatched_key_priorities());
    let ordered = ordered_key_ids(&snapshot, ProviderPriorityMode::Provider);

    assert_eq!(ordered, vec!["key-internal-first", "key-global-first"]);
}

#[test]
fn order_candidate_parts_key_mode_uses_global_priority_snapshot() {
    let snapshot = snapshot_with_provider(provider_with_mismatched_key_priorities());
    let ordered = ordered_key_ids(&snapshot, ProviderPriorityMode::Key);

    assert_eq!(ordered, vec!["key-global-first", "key-internal-first"]);
}

fn ordered_key_ids(snapshot: &crate::llm_proxy::cache::snapshot::SchedulingSnapshot, priority_mode: ProviderPriorityMode) -> Vec<String> {
    let group = &snapshot.groups[0];
    let token = api_token(ApiTokenType::Independent, None);
    let request = request();
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request,
        affinity: None,
        scheduling_mode: snapshot.scheduling_mode,
        request_id: "request-1",
        cooled_provider_ids: &Default::default(),
    });
    order_candidate_parts(OrderCandidatePartsInput {
        parts,
        token: &token,
        group,
        user_access: None,
        request,
        model_id: "model-a",
        request_id: "request-1",
        affinity: None,
        mode: snapshot.scheduling_mode,
        priority_mode,
    })
    .unwrap()
    .into_iter()
    .map(|part| part.keys[0].id.clone())
    .collect()
}

fn provider_with_mismatched_key_priorities() -> crate::llm_proxy::cache::snapshot::CachedProvider {
    let provider = provider_with_endpoints_and_keys();
    crate::llm_proxy::cache::snapshot::CachedProvider {
        keys: vec![
            key_with_global_priority("key-internal-first", 1, 100),
            key_with_global_priority("key-global-first", 100, 1),
        ],
        ..provider
    }
}

fn key_with_global_priority(id: &str, internal_priority: i32, global_priority: i32) -> crate::llm_proxy::cache::snapshot::CachedProviderKey {
    let mut key = super::helpers::provider_key(id, internal_priority, vec!["openai:chat"]);
    key.global_priority = global_priority;
    key
}
