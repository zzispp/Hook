use std::collections::HashSet;

use types::provider::ProviderSchedulingMode;

use super::helpers::{provider_with_endpoints_and_keys, request, snapshot_with_provider};
use crate::llm_proxy::candidate::selection::matching::{MatchingCandidatePartsInput, matching_candidate_parts};

#[test]
fn matching_candidate_parts_excludes_inactive_provider_from_proxy_routing() {
    let snapshot = snapshot_with_provider(crate::llm_proxy::cache::snapshot::CachedProvider {
        is_active: false,
        ..provider_with_endpoints_and_keys()
    });
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

    assert_eq!(parts.len(), 0);
}
