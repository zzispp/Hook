use proxy::{
    format_conversion::ApiFormat,
    scheduler::{AffinityCandidate, AttemptOutcome, CandidateBuilder, FailoverExecutor, PriorityMode, SchedulerError, SchedulerInput, SchedulingMode},
};

mod common;
use common::{base_input, provider_a, provider_b, provider_with_gemini_low_priority, provider_with_priority, provider_with_two_keys};
use proxy::scheduler::{EndpointSnapshot, KeySnapshot, ModelBindingSnapshot, ProviderSnapshot};

#[test]
fn scheduler_filters_by_endpoint_format_and_model() {
    let input = SchedulerInput {
        providers: vec![provider_a(), provider_b()],
        ..base_input()
    };

    let candidates = CandidateBuilder::build(&input).unwrap();

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].provider_id, "provider-a");
    assert_eq!(candidates[0].provider_model_name, "upstream-gpt-4o-mini");
    assert_eq!(candidates[0].provider_api_format, ApiFormat::OpenAiChat);
    assert!(!candidates[0].needs_conversion);
}

#[test]
fn scheduler_rejects_user_disallowed_model() {
    let input = SchedulerInput {
        user_allowed_model_ids: vec!["model-b".into()],
        providers: vec![provider_a()],
        ..base_input()
    };

    let error = CandidateBuilder::build(&input).unwrap_err();

    assert_eq!(error, SchedulerError::UserModelDenied { model: "gpt-4o-mini".into() });
}

#[test]
fn scheduler_filters_by_user_provider_scope() {
    let input = SchedulerInput {
        user_allowed_provider_ids: vec!["provider-b".into()],
        providers: vec![provider_a()],
        ..base_input()
    };

    let error = CandidateBuilder::build(&input).unwrap_err();

    assert_eq!(error, SchedulerError::NoModelCandidate { model: "gpt-4o-mini".into() });
}

#[test]
fn scheduler_filters_by_group_provider_key_scope() {
    let input = SchedulerInput {
        group_allowed_provider_key_ids: Some(vec!["key-a-2".into()]),
        providers: vec![provider_with_two_keys()],
        ..base_input()
    };

    let candidates = CandidateBuilder::build(&input).unwrap();

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].provider_id, "provider-a");
    assert_eq!(candidates[0].key_id, "key-a-2");
}

#[test]
fn scheduler_rejects_empty_group_key_scope() {
    let input = SchedulerInput {
        group_allowed_provider_key_ids: Some(Vec::new()),
        providers: vec![provider_a()],
        ..base_input()
    };

    let error = CandidateBuilder::build(&input).unwrap_err();

    assert_eq!(error, SchedulerError::NoModelCandidate { model: "gpt-4o-mini".into() });
}

#[test]
fn scheduler_returns_group_model_unavailable_when_no_provider_model_matches() {
    let input = SchedulerInput {
        requested_model_id: "claude-3-haiku".into(),
        providers: vec![provider_a()],
        ..base_input()
    };

    let error = CandidateBuilder::build(&input).unwrap_err();

    assert_eq!(
        error,
        SchedulerError::NoModelCandidate {
            model: "claude-3-haiku".into(),
        }
    );
}

#[test]
fn scheduler_demotes_conversion_unless_provider_keeps_priority() {
    let input = SchedulerInput {
        providers: vec![provider_with_gemini_low_priority(), provider_a()],
        ..base_input()
    };

    let candidates = CandidateBuilder::build(&input).unwrap();

    assert_eq!(candidates[0].provider_id, "provider-a");
    assert!(!candidates[0].needs_conversion);
    assert_eq!(candidates[1].provider_id, "provider-gemini");
    assert!(candidates[1].needs_conversion);
}

#[test]
fn scheduler_provider_priority_mode_orders_by_provider_then_key() {
    let candidates = CandidateBuilder::build(&SchedulerInput {
        providers: vec![
            provider_with_priority("provider-slow-key", 1, "key-slow", 90),
            provider_with_priority("provider-fast-key", 10, "key-fast", 1),
        ],
        priority_mode: PriorityMode::Provider,
        ..base_input()
    })
    .unwrap();

    assert_eq!(candidates[0].provider_id, "provider-slow-key");
    assert_eq!(candidates[0].key_id, "key-slow");
}

#[test]
fn scheduler_key_priority_mode_orders_by_key_then_provider() {
    let candidates = CandidateBuilder::build(&SchedulerInput {
        providers: vec![
            provider_with_priority("provider-slow-key", 1, "key-slow", 90),
            provider_with_priority("provider-fast-key", 10, "key-fast", 1),
        ],
        priority_mode: PriorityMode::Key,
        ..base_input()
    })
    .unwrap();

    assert_eq!(candidates[0].provider_id, "provider-fast-key");
    assert_eq!(candidates[0].key_id, "key-fast");
}

#[test]
fn scheduler_key_priority_mode_uses_endpoint_format_bucket() {
    let provider = provider_with_format_priority_keys();
    let openai = CandidateBuilder::build(&SchedulerInput {
        providers: vec![provider.clone()],
        priority_mode: PriorityMode::Key,
        client_format: ApiFormat::OpenAiChat,
        ..base_input()
    })
    .unwrap();
    let gemini = CandidateBuilder::build(&SchedulerInput {
        providers: vec![provider],
        priority_mode: PriorityMode::Key,
        client_format: ApiFormat::GeminiChat,
        ..base_input()
    })
    .unwrap();

    assert_eq!(openai[0].key_id, "key-openai-first");
    assert_eq!(openai[0].endpoint_id, "endpoint-openai");
    assert_eq!(gemini[0].key_id, "key-gemini-first");
    assert_eq!(gemini[0].endpoint_id, "endpoint-gemini");
}

#[test]
fn scheduler_cache_affinity_promotes_matching_key() {
    let input = SchedulerInput {
        affinity: Some(AffinityCandidate {
            provider_id: "provider-a".into(),
            endpoint_id: "endpoint-a-openai".into(),
            key_id: "key-a-2".into(),
        }),
        scheduling_mode: SchedulingMode::CacheAffinity,
        providers: vec![provider_with_two_keys()],
        ..base_input()
    };

    let candidates = CandidateBuilder::build(&input).unwrap();

    assert_eq!(candidates[0].key_id, "key-a-2");
    assert!(candidates[0].is_cached);
}

#[test]
fn scheduler_cache_affinity_requires_provider_endpoint_and_key_match() {
    let input = SchedulerInput {
        affinity: Some(AffinityCandidate {
            provider_id: "provider-b".into(),
            endpoint_id: "endpoint-a-openai".into(),
            key_id: "key-a-2".into(),
        }),
        scheduling_mode: SchedulingMode::CacheAffinity,
        providers: vec![provider_with_two_keys()],
        ..base_input()
    };

    let candidates = CandidateBuilder::build(&input).unwrap();

    assert!(candidates.iter().all(|candidate| !candidate.is_cached));
}

#[test]
fn scheduler_cache_affinity_without_cached_key_uses_request_seed() {
    let first = CandidateBuilder::build(&SchedulerInput {
        scheduling_mode: SchedulingMode::CacheAffinity,
        load_balance_seed: Some("request-1".into()),
        providers: vec![provider_with_two_keys()],
        ..base_input()
    })
    .unwrap();
    let second = CandidateBuilder::build(&SchedulerInput {
        scheduling_mode: SchedulingMode::CacheAffinity,
        load_balance_seed: Some("request-2".into()),
        providers: vec![provider_with_two_keys()],
        ..base_input()
    })
    .unwrap();

    assert_eq!(first[0].key_id, "key-a-2");
    assert_eq!(second[0].key_id, "key-a-1");
}

#[test]
fn scheduler_load_balance_keeps_priority_group_and_uses_stable_hash() {
    let input = SchedulerInput {
        scheduling_mode: SchedulingMode::LoadBalance,
        load_balance_seed: Some("request-1".into()),
        providers: vec![provider_with_two_keys()],
        ..base_input()
    };

    let first = CandidateBuilder::build(&input).unwrap();
    let second = CandidateBuilder::build(&input).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].provider_id, "provider-a");
    assert_eq!(first[1].provider_id, "provider-a");
}

#[test]
fn scheduler_key_priority_load_balance_keeps_key_priority_tier() {
    let candidates = CandidateBuilder::build(&SchedulerInput {
        scheduling_mode: SchedulingMode::LoadBalance,
        priority_mode: PriorityMode::Key,
        load_balance_seed: Some("request-1".into()),
        providers: vec![
            provider_with_priority("provider-high", 1, "key-high", 20),
            provider_with_priority("provider-a", 10, "key-a", 5),
            provider_with_priority("provider-b", 20, "key-b", 5),
        ],
        ..base_input()
    })
    .unwrap();

    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].key_id, "key-a");
    assert_eq!(candidates[1].key_id, "key-b");
    assert_eq!(candidates[2].key_id, "key-high");
}

#[test]
fn scheduler_load_balance_keeps_conversion_demoted() {
    let input = SchedulerInput {
        scheduling_mode: SchedulingMode::LoadBalance,
        load_balance_seed: Some("request-1".into()),
        providers: vec![provider_with_gemini_low_priority(), provider_a()],
        ..base_input()
    };

    let candidates = CandidateBuilder::build(&input).unwrap();

    assert!(!candidates[0].needs_conversion);
    assert!(candidates.iter().skip(1).any(|candidate| candidate.needs_conversion));
}

#[test]
fn scheduler_failover_stops_after_retryable_failure_then_success() {
    let candidates = CandidateBuilder::build(&SchedulerInput {
        providers: vec![provider_with_two_keys()],
        ..base_input()
    })
    .unwrap();

    let attempts = FailoverExecutor::execute(&candidates, |candidate| {
        if candidate.key_id == "key-a-1" {
            AttemptOutcome::RetryableFailure("rate_limit".into())
        } else {
            AttemptOutcome::Success
        }
    })
    .unwrap();

    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].candidate.key_id, "key-a-1");
    assert_eq!(attempts[1].candidate.key_id, "key-a-2");
    assert_eq!(attempts[1].outcome, AttemptOutcome::Success);
}

fn provider_with_format_priority_keys() -> ProviderSnapshot {
    ProviderSnapshot {
        id: "provider-format".into(),
        name: "Provider Format".into(),
        priority: 10,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        endpoints: vec![
            endpoint("endpoint-openai", ApiFormat::OpenAiChat),
            endpoint("endpoint-gemini", ApiFormat::GeminiChat),
        ],
        keys: vec![
            key_with_priorities("key-openai-first", 10, 1, 90),
            key_with_priorities("key-gemini-first", 20, 90, 1),
        ],
        models: vec![ModelBindingSnapshot {
            global_model_id: "gpt-4o-mini".into(),
            provider_model_name: "upstream-format".into(),
        }],
    }
}

fn endpoint(id: &str, api_format: ApiFormat) -> EndpointSnapshot {
    EndpointSnapshot {
        id: id.into(),
        api_format,
        is_active: true,
        accepts_format_conversion: true,
        supports_stream_conversion: true,
    }
}

fn key_with_priorities(id: &str, internal_priority: i32, openai_priority: i32, gemini_priority: i32) -> KeySnapshot {
    KeySnapshot {
        id: id.into(),
        api_formats: vec![ApiFormat::OpenAiChat, ApiFormat::GeminiChat],
        internal_priority,
        global_priority_by_format: std::collections::BTreeMap::from([(ApiFormat::OpenAiChat, openai_priority), (ApiFormat::GeminiChat, gemini_priority)]),
        cache_ttl_minutes: 5,
        is_active: true,
    }
}
