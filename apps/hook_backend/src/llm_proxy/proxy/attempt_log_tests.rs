use std::sync::{Arc, Mutex};

use rust_decimal::Decimal;
use types::{model::PatchField, model::TieredPricingConfig};

use super::{
    AttemptCancelGuard, AttemptCancelHandle, AttemptCancelPhase, AttemptCancelShared, cancelled_after_upstream_response_record,
    cancelled_before_upstream_response_record, stream_candidate_watchdog_timeout_record, take_cancel_phase,
};
use crate::llm_proxy::{
    audit::{AttemptAuditInput, request_billing_status},
    candidate::{CandidateRoute, CandidateTrace, ProxyCandidate},
};

#[test]
fn disarm_keeps_guard_owned_by_caller() {
    fn accepts_shared_disarm(_: fn(&AttemptCancelGuard)) {}

    accepts_shared_disarm(AttemptCancelGuard::disarm);
}

#[test]
fn armed_awaiting_terminal_guard_records_cancel_phase() {
    let shared = cancel_shared(AttemptCancelPhase::AwaitingTerminal, true);

    assert!(matches!(take_cancel_phase(&shared), Some(AttemptCancelPhase::AwaitingTerminal)));
}

#[test]
fn disarmed_awaiting_terminal_guard_records_no_cancel_phase() {
    let shared = cancel_shared(AttemptCancelPhase::AwaitingTerminal, true);
    let handle = AttemptCancelHandle { shared: Arc::clone(&shared) };

    handle.disarm();

    assert!(take_cancel_phase(&shared).is_none());
}

#[test]
fn cancelled_before_upstream_response_is_explicit_terminal_record() {
    let candidate = candidate();
    let input = cancelled_before_upstream_response_record(&candidate, 0, 42);

    assert_eq!(input.status, "cancelled");
    assert!(input.finished);
    assert_eq!(input.status_code, Some(499));
    assert_eq!(input.latency_ms, Some(42));
    assert_eq!(input.error_type, Some("client_disconnected"));
    assert_eq!(input.error_message, Some("client disconnected before upstream response started"));
    assert_eq!(input.termination_origin, PatchField::Value("client".into()));
    assert_eq!(input.termination_reason, PatchField::Value("disconnected".into()));
    assert_eq!(input.stream_end_reason, PatchField::Value("client_gone".into()));
    assert_eq!(request_billing_status(&AttemptAuditInput::from(input), None), "void");
}

#[test]
fn cancelled_after_upstream_response_is_explicit_terminal_record() {
    let candidate = candidate();
    let input = cancelled_after_upstream_response_record(&candidate, 0, 42);

    assert_eq!(input.status, "cancelled");
    assert!(input.finished);
    assert_eq!(input.status_code, Some(499));
    assert_eq!(input.latency_ms, Some(42));
    assert_eq!(input.error_type, Some("client_disconnected"));
    assert_eq!(input.error_message, Some("client disconnected before request terminal finalization"));
    assert_eq!(input.termination_origin, PatchField::Value("client".into()));
    assert_eq!(input.termination_reason, PatchField::Value("disconnected".into()));
    assert_eq!(input.stream_end_reason, PatchField::Value("client_gone".into()));
    assert_eq!(request_billing_status(&AttemptAuditInput::from(input), None), "void");
}

#[test]
fn stream_candidate_watchdog_timeout_is_explicit_failed_record() {
    let candidate = candidate();
    let input = stream_candidate_watchdog_timeout_record(&candidate, 0, 42);

    assert_eq!(input.status, "failed");
    assert!(input.finished);
    assert_eq!(input.status_code, Some(504));
    assert_eq!(input.latency_ms, Some(42));
    assert_eq!(input.error_type, Some("local_stream_candidate_watchdog_timeout"));
    assert_eq!(input.error_message, Some("stream candidate timed out before handoff completed"));
    assert_eq!(request_billing_status(&AttemptAuditInput::from(input), None), "void");
}

fn cancel_shared(phase: AttemptCancelPhase, armed: bool) -> Arc<Mutex<AttemptCancelShared>> {
    Arc::new(Mutex::new(AttemptCancelShared { phase, armed }))
}

fn candidate() -> ProxyCandidate {
    ProxyCandidate {
        trace: CandidateTrace {
            token_id: Some("token-1".into()),
            user_id_snapshot: Some("user-1".into()),
            username_snapshot: Some("admin".into()),
            token_name_snapshot: Some("token".into()),
            token_prefix_snapshot: Some("sk".into()),
            group_code: Some("default".into()),
            global_model_id: "model-1".into(),
            provider_model_id: "provider-model-1".into(),
            model_name_snapshot: "gpt-test".into(),
            provider_id: "provider-1".into(),
            provider_name_snapshot: "provider".into(),
            endpoint_id: "endpoint-1".into(),
            endpoint_name_snapshot: "openai_cli".into(),
            key_id: "key-1".into(),
            key_name_snapshot: "key".into(),
            key_preview_snapshot: "***key".into(),
            client_api_format: "openai_cli".into(),
            provider_api_format: "openai_cli".into(),
            needs_conversion: false,
            is_stream: true,
            is_cached: false,
            candidate_index: 0,
        },
        requested_model_name: "gpt-test".into(),
        api_key: "secret".into(),
        base_url: "https://example.com".into(),
        custom_path: None,
        upstream_url: "https://example.com/v1/responses".into(),
        provider_model_name: "gpt-test".into(),
        reasoning_effort: None,
        header_rules: None,
        body_rules: None,
        price_per_request: None,
        tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        billing_multiplier: Decimal::ONE,
        max_retries: 0,
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(30.0),
        stream_idle_timeout_seconds: Some(30.0),
        cache_ttl_minutes: 5,
        key_rpm_limit: None,
        is_cached: false,
        route: CandidateRoute { options: Vec::new() },
    }
}
