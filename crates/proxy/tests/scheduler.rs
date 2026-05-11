use proxy::{
    format_conversion::ApiFormat,
    scheduler::{AttemptOutcome, CandidateBuilder, FailoverExecutor, SchedulerError, SchedulerInput, SchedulingMode},
};

mod common;
use common::{base_input, provider_a, provider_b, provider_with_gemini_low_priority, provider_with_two_keys};

#[test]
fn scheduler_filters_by_group_provider_endpoint_format_and_model() {
    let input = SchedulerInput {
        group_allowed_provider_ids: vec!["provider-a".into()],
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
fn scheduler_cache_affinity_promotes_matching_key() {
    let input = SchedulerInput {
        affinity_key: Some("key-a-2".into()),
        scheduling_mode: SchedulingMode::CacheAffinity,
        providers: vec![provider_with_two_keys()],
        ..base_input()
    };

    let candidates = CandidateBuilder::build(&input).unwrap();

    assert_eq!(candidates[0].key_id, "key-a-2");
    assert!(candidates[0].is_cached);
}

#[test]
fn scheduler_load_balance_keeps_priority_group_and_uses_stable_hash() {
    let input = SchedulerInput {
        scheduling_mode: SchedulingMode::LoadBalance,
        providers: vec![provider_with_two_keys()],
        ..base_input()
    };

    let first = CandidateBuilder::build(&input).unwrap();
    let second = CandidateBuilder::build(&input).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].provider_priority, first[1].provider_priority);
    assert_eq!(first[0].key_priority, first[1].key_priority);
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
