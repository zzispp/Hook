use types::provider::RoutingProfileId;

use super::helpers::{provider_with_endpoints_and_keys, snapshot_with_provider};
use crate::llm_proxy::{
    LlmProxyError,
    candidate::selection::{effective_routing_profile_id, ranking_context_from_snapshot, request_id_seed, validate_rankings_request},
};

#[test]
fn routing_request_id_seed_reuses_provided_seed() {
    let seed = request_id_seed(Some("stable-seed".into())).unwrap();

    assert_eq!(seed, "stable-seed");
}

#[test]
fn routing_request_id_seed_rejects_empty_seed() {
    let error = request_id_seed(Some(" ".into())).unwrap_err();

    assert!(matches!(error, LlmProxyError::InvalidRequest(message) if message == "request_id_seed cannot be empty"));
}

#[test]
fn routing_rankings_request_requires_group_model_and_format() {
    let request = types::provider::RoutingRankingsRequest {
        group_code: String::new(),
        model: "gpt-test".into(),
        api_format: "openai:chat".into(),
        is_stream: false,
        window: Default::default(),
        include_excluded: false,
        request_id_seed: None,
    };

    let error = validate_rankings_request(&request).unwrap_err();

    assert!(matches!(error, LlmProxyError::InvalidRequest(message) if message == "routing rankings requires group_code"));
}

#[test]
fn routing_effective_profile_prefers_model_then_group_then_default() {
    let mut snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let mut model = super::super::resolve_global_model(&snapshot, "gpt-test").unwrap();

    assert_eq!(effective_routing_profile_id(&snapshot.groups[0], &model), RoutingProfileId::Balanced);

    snapshot.groups[0].routing_profile_id = Some(RoutingProfileId::HighAvailability);
    assert_eq!(effective_routing_profile_id(&snapshot.groups[0], &model), RoutingProfileId::HighAvailability);

    model.routing_profile_id = Some(RoutingProfileId::FirstByte);
    assert_eq!(effective_routing_profile_id(&snapshot.groups[0], &model), RoutingProfileId::FirstByte);
}

#[test]
fn ranking_context_uses_group_code_context() {
    let snapshot = snapshot_with_provider(provider_with_endpoints_and_keys());
    let context = ranking_context_from_snapshot(&snapshot, "default", super::helpers::request(), "seed-1", &std::collections::HashSet::new()).unwrap();

    assert_eq!(context.group.code, "default");
    assert_eq!(context.global_model.name, "gpt-test");
    assert_eq!(context.routing_profile_id, RoutingProfileId::Balanced);
    assert!(!context.parts.is_empty());
}
