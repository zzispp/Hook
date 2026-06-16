use std::collections::BTreeMap;

use types::{
    api_token::ApiTokenType,
    provider::{ProviderPriorityMode, ProviderSchedulingMode},
};

use super::helpers::{api_token, provider_b, provider_key, provider_with_endpoints_and_keys, provider_with_keys, request, snapshot_with_provider};
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

#[test]
fn order_candidate_parts_uses_group_scoped_provider_priority() {
    let mut snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    snapshot.providers.push(provider_b());
    snapshot.groups[0].provider_priorities = BTreeMap::from([("provider-a".to_owned(), 100), ("provider-b".to_owned(), 1)]);

    let ordered = ordered_key_ids(&snapshot, ProviderPriorityMode::Provider);

    assert_eq!(ordered[0], "key-b-1");
}

#[test]
fn order_candidate_parts_uses_group_scoped_key_priority() {
    let mut snapshot = snapshot_with_provider(provider_with_mismatched_key_priorities());
    snapshot.groups[0].provider_key_priorities = BTreeMap::from([("key-internal-first".to_owned(), 100), ("key-global-first".to_owned(), 1)]);

    let ordered = ordered_key_ids(&snapshot, ProviderPriorityMode::Provider);

    assert_eq!(ordered, vec!["key-global-first", "key-internal-first"]);
}

#[test]
fn routing_order_candidate_parts_load_balance_seed_is_stable() {
    let provider = provider_with_keys(vec![
        provider_key("key-a-1", 10, vec!["openai:chat"]),
        provider_key("key-a-2", 20, vec!["openai:chat"]),
        provider_key("key-a-3", 30, vec!["openai:chat"]),
    ]);
    let mut snapshot = snapshot_with_provider(provider);
    snapshot.scheduling_mode = ProviderSchedulingMode::LoadBalance;

    let first = ordered_key_ids_with_seed(&snapshot, ProviderPriorityMode::Provider, "request-seed-a");
    let second = ordered_key_ids_with_seed(&snapshot, ProviderPriorityMode::Provider, "request-seed-a");

    assert_eq!(first, second);
}

fn ordered_key_ids(snapshot: &crate::llm_proxy::cache::snapshot::SchedulingSnapshot, priority_mode: ProviderPriorityMode) -> Vec<String> {
    ordered_key_ids_with_seed(snapshot, priority_mode, "request-1")
}

fn ordered_key_ids_with_seed(
    snapshot: &crate::llm_proxy::cache::snapshot::SchedulingSnapshot,
    priority_mode: ProviderPriorityMode,
    request_id: &str,
) -> Vec<String> {
    let group = &snapshot.groups[0];
    let token = api_token(ApiTokenType::Independent, None);
    let request = request();
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request: request.clone(),
        affinity: None,
        scheduling_mode: snapshot.scheduling_mode,
        request_id,
        cooled_provider_ids: &Default::default(),
    });
    order_candidate_parts(OrderCandidatePartsInput {
        parts,
        token: &token,
        group,
        user_access: None,
        request,
        model_id: "model-a",
        request_id,
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
            key_with_format_priority("key-internal-first", 1, "openai:chat", 100),
            key_with_format_priority("key-global-first", 100, "openai:chat", 1),
        ],
        ..provider
    }
}

fn key_with_format_priority(id: &str, internal_priority: i32, api_format: &str, global_priority: i32) -> crate::llm_proxy::cache::snapshot::CachedProviderKey {
    let mut key = super::helpers::provider_key(id, internal_priority, vec!["openai:chat"]);
    key.global_priority_by_format.insert(api_format.to_owned(), global_priority);
    key
}
