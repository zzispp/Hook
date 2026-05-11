use proxy::scheduler::{AttemptOutcome, CandidateAuditInput, CandidateAuditRecorder, CandidateBuilder, FailoverExecutor, ModelAccessPolicy, SchedulerInput};

mod common;
use common::{base_input, provider_a, provider_with_two_keys};

#[test]
fn request_candidate_audit_records_available_candidates() {
    let candidates = CandidateBuilder::build(&SchedulerInput {
        providers: vec![provider_with_two_keys()],
        ..base_input()
    })
    .unwrap();

    let records = CandidateAuditRecorder::available_records(audit_input(), &candidates);

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].status, "available");
    assert_eq!(records[0].request_id, "req-1");
    assert_eq!(records[0].token_id.as_deref(), Some("token-1"));
    assert_eq!(records[0].provider_id.as_deref(), Some("provider-a"));
    assert_eq!(records[0].client_api_format, "openai_chat");
    assert!(!records[0].started);
    assert!(!records[0].finished);
}

#[test]
fn request_candidate_audit_records_failed_then_success_attempts() {
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

    let records = CandidateAuditRecorder::attempt_records(audit_input(), &attempts);

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].status, "failed");
    assert_eq!(records[0].error_type.as_deref(), Some("retryable_upstream_error"));
    assert_eq!(records[0].error_message.as_deref(), Some("rate_limit"));
    assert_eq!(records[1].status, "success");
    assert_eq!(records[1].status_code, Some(200));
    assert_eq!(records[1].error_type, None);
    assert!(records[1].started);
    assert!(records[1].finished);
}

#[test]
fn request_candidate_audit_records_no_candidate() {
    let record = CandidateAuditRecorder::no_candidate_record(
        CandidateAuditInput {
            global_model_id: Some("missing-model"),
            ..audit_input()
        },
        "该分组下暂无 missing-model 模型可用".into(),
    );

    assert_eq!(record.status, "failed");
    assert_eq!(record.provider_id, None);
    assert_eq!(record.endpoint_id, None);
    assert_eq!(record.key_id, None);
    assert_eq!(record.error_type.as_deref(), Some("no_candidate"));
    assert_eq!(record.error_message.as_deref(), Some("该分组下暂无 missing-model 模型可用"));
    assert!(record.finished);
}

#[test]
fn request_candidate_audit_preserves_model_scope_from_candidate() {
    let candidates = CandidateBuilder::build(&SchedulerInput {
        token_model_policy: ModelAccessPolicy::Limited(vec!["gpt-4o-mini".into()]),
        providers: vec![provider_a()],
        ..base_input()
    })
    .unwrap();

    let records = CandidateAuditRecorder::available_records(
        CandidateAuditInput {
            global_model_id: None,
            ..audit_input()
        },
        &candidates,
    );

    assert_eq!(records[0].global_model_id.as_deref(), Some("gpt-4o-mini"));
    assert_eq!(records[0].provider_api_format.as_deref(), Some("OpenAiChat"));
    assert!(!records[0].needs_conversion);
}

fn audit_input() -> CandidateAuditInput<'static> {
    CandidateAuditInput {
        request_id: "req-1",
        token_id: Some("token-1"),
        group_code: Some("default"),
        global_model_id: Some("gpt-4o-mini"),
        client_api_format: "openai_chat",
        is_stream: false,
    }
}
