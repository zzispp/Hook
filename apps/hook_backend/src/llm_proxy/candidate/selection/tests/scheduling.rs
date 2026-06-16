use std::collections::BTreeMap;

use types::{
    api_token::ApiTokenType,
    provider::{ProviderPriorityMode, ProviderSchedulingMode},
};

use super::super::{EffectiveCacheAffinityMode, mark_score_bonus_affinity};
use super::helpers::{api_token, provider_b, provider_with_endpoints_and_keys, request, snapshot_with_provider};
use crate::llm_proxy::AffinitySelection;
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
fn score_bonus_affinity_marks_candidate_without_reordering() {
    let mut snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    snapshot.scheduling_mode = ProviderSchedulingMode::CacheAffinity;
    let group = &snapshot.groups[0];
    let token = api_token(ApiTokenType::Independent, None);
    let request = request();
    let affinity = affinity("key-a-2");
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request,
        affinity: None,
        scheduling_mode: snapshot.scheduling_mode,
        request_id: "request-1",
        cooled_provider_ids: &Default::default(),
    });
    let ordered = order_candidate_parts(OrderCandidatePartsInput {
        parts,
        token: &token,
        group,
        user_access: None,
        request,
        model_id: "model-a",
        request_id: "request-1",
        affinity: None,
        mode: snapshot.scheduling_mode,
        priority_mode: ProviderPriorityMode::Provider,
    })
    .unwrap();
    let base_order = key_ids(&ordered);

    let marked = mark_score_bonus_affinity(ordered, Some(&affinity), EffectiveCacheAffinityMode::ScoreBonus);

    assert_eq!(key_ids(&marked), base_order);
    let marked_part = marked.iter().find(|part| part.keys[0].id == "key-a-2").expect("affinity key candidate");
    assert!(marked_part.affinity_bonus);
    assert!(!marked_part.is_cached);
}

#[test]
fn disabled_affinity_does_not_mark_candidate() {
    let parts = vec![part_for_affinity_test("key-a-2")];
    let affinity = affinity("key-a-2");

    let marked = mark_score_bonus_affinity(parts, Some(&affinity), EffectiveCacheAffinityMode::Disabled);

    assert!(!marked[0].affinity_bonus);
    assert!(!marked[0].is_cached);
}

#[test]
fn prefer_cached_affinity_preserves_scheduler_promotion() {
    let mut snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    snapshot.scheduling_mode = ProviderSchedulingMode::CacheAffinity;
    let group = &snapshot.groups[0];
    let token = api_token(ApiTokenType::Independent, None);
    let request = request();
    let affinity = affinity("key-a-2");
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group,
        user_access: None,
        model_id: "model-a",
        request,
        affinity: Some(&affinity),
        scheduling_mode: snapshot.scheduling_mode,
        request_id: "request-1",
        cooled_provider_ids: &Default::default(),
    });

    let ordered = order_candidate_parts(OrderCandidatePartsInput {
        parts,
        token: &token,
        group,
        user_access: None,
        request,
        model_id: "model-a",
        request_id: "request-1",
        affinity: Some(affinity),
        mode: snapshot.scheduling_mode,
        priority_mode: ProviderPriorityMode::Provider,
    })
    .unwrap();

    assert_eq!(ordered[0].keys[0].id, "key-a-2");
    assert!(ordered[0].is_cached);
    assert!(ordered[0].affinity_bonus);
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

fn key_ids(parts: &[crate::llm_proxy::candidate::selection::CandidateParts]) -> Vec<String> {
    parts.iter().map(|part| part.keys[0].id.clone()).collect()
}

fn affinity(key_id: &str) -> AffinitySelection {
    AffinitySelection {
        provider_id: "provider-a".into(),
        endpoint_id: "endpoint-openai".into(),
        key_id: key_id.into(),
    }
}

fn part_for_affinity_test(key_id: &str) -> crate::llm_proxy::candidate::selection::CandidateParts {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let request = request();
    let parts = matching_candidate_parts(MatchingCandidatePartsInput {
        snapshot: &snapshot,
        group: &snapshot.groups[0],
        user_access: None,
        model_id: "model-a",
        request,
        affinity: None,
        scheduling_mode: snapshot.scheduling_mode,
        request_id: "request-1",
        cooled_provider_ids: &Default::default(),
    });
    parts.into_iter().find(|part| part.keys[0].id == key_id).expect("candidate part should exist")
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
