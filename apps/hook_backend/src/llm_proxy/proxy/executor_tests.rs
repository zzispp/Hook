use std::time::Duration;

use axum::http::StatusCode;
use rust_decimal::Decimal;
use types::model::TieredPricingConfig;

use super::{
    AttemptExecutionState, AttemptOnceOutcome, StreamWatchdogOutcome, is_openai_cli_to_chat_candidate, probe_slot_timeout_outcome,
    run_stream_candidate_watchdog, should_skip_codex_history_after_error, should_skip_codex_history_candidate, stream_candidate_watchdog_timeout_output,
    stream_pre_output_failure_task_output, stream_send_error_last_failure, stream_send_error_outcome,
};
use crate::llm_proxy::{
    LlmProxyError,
    candidate::{CandidateRoute, CandidateTrace, ProxyCandidate},
    proxy::{attempt_log::AttemptCancelHandle, stream_transport::StreamPreOutputFailure},
};

#[tokio::test]
async fn stream_candidate_watchdog_returns_completed_result_before_timeout() {
    let outcome = run_stream_candidate_watchdog(Some(Duration::from_millis(50)), AttemptCancelHandle::noop_for_test(), async {
        Ok::<_, LlmProxyError>(7_i32)
    })
    .await
    .expect("watchdog should finish");

    assert!(matches!(outcome, StreamWatchdogOutcome::Completed(7)));
}

#[tokio::test]
async fn stream_candidate_watchdog_times_out_pending_task() {
    let outcome = run_stream_candidate_watchdog(Some(Duration::from_millis(5)), AttemptCancelHandle::noop_for_test(), async {
        std::future::pending::<Result<i32, LlmProxyError>>().await
    })
    .await
    .expect("watchdog timeout should be handled");

    assert!(matches!(outcome, StreamWatchdogOutcome::TimedOut));
}

#[test]
fn stream_candidate_watchdog_timeout_advances_to_next_candidate() {
    let output = stream_candidate_watchdog_timeout_output();

    assert!(matches!(output.outcome, AttemptOnceOutcome::NextCandidate));
    assert!(output.last_failure.is_some());
    assert_eq!(
        output.last_error.map(|error| error.to_string()),
        Some("stream candidate watchdog timed out".into())
    );
}

#[test]
fn probe_slot_timeout_continues_candidate_route() {
    let output = probe_slot_timeout_outcome();

    assert!(matches!(output, AttemptOnceOutcome::ContinueCandidate));
}

#[test]
fn stream_preoutput_failure_continues_candidate_with_last_failure() {
    let output = stream_pre_output_failure_task_output(StreamPreOutputFailure {
        status: StatusCode::BAD_GATEWAY,
        error_type: "upstream_stream_incomplete",
        message: "upstream stream ended without a terminal event".into(),
        advance_candidate: false,
    });

    assert!(matches!(output.outcome, AttemptOnceOutcome::ContinueCandidate));
    assert!(output.last_failure.is_some());
    assert_eq!(
        output.last_error.map(|error| error.to_string()),
        Some("upstream_stream_incomplete: upstream stream ended without a terminal event".into())
    );
}

#[test]
fn stream_preoutput_first_byte_timeout_advances_to_next_candidate() {
    let output = stream_pre_output_failure_task_output(StreamPreOutputFailure {
        status: StatusCode::GATEWAY_TIMEOUT,
        error_type: "first_byte_timeout",
        message: "stream first byte timeout".into(),
        advance_candidate: true,
    });

    assert!(matches!(output.outcome, AttemptOnceOutcome::NextCandidate));
    assert!(output.last_failure.is_some());
    assert_eq!(
        output.last_error.map(|error| error.to_string()),
        Some("first_byte_timeout: stream first byte timeout".into())
    );
}

#[test]
fn stream_timeout_send_error_advances_to_next_candidate() {
    let outcome = stream_send_error_outcome(&req::ClientError::Timeout, None);

    assert!(matches!(outcome, AttemptOnceOutcome::NextCandidate));
    assert!(stream_send_error_last_failure(&req::ClientError::Timeout).is_some());
}

#[test]
fn non_timeout_send_error_keeps_current_candidate_retry_path() {
    let outcome = stream_send_error_outcome(&req::ClientError::Network("reset".into()), None);

    assert!(matches!(outcome, AttemptOnceOutcome::ContinueCandidate));
    assert!(stream_send_error_last_failure(&req::ClientError::Network("reset".into())).is_none());
}

#[test]
fn codex_history_skip_matches_openai_cli_to_chat_only() {
    let mut state = AttemptExecutionState {
        codex_chat_history_unavailable_message: Some("missing Codex chat history".into()),
    };
    let chat = candidate("openai:cli", "openai:chat");
    let cli = candidate("openai:cli", "openai:cli");
    let chat_client = candidate("openai:chat", "openai:chat");

    assert!(is_openai_cli_to_chat_candidate(&chat));
    assert!(should_skip_codex_history_candidate(&state, &chat));
    assert!(!should_skip_codex_history_candidate(&state, &cli));
    assert!(!should_skip_codex_history_candidate(&state, &chat_client));

    state.codex_chat_history_unavailable_message = None;
    assert!(!should_skip_codex_history_candidate(&state, &chat));
}

#[test]
fn codex_history_error_only_triggers_skip_for_cli_to_chat() {
    let error = LlmProxyError::CodexChatHistoryUnavailable("missing Codex chat history".into());
    let invalid = LlmProxyError::InvalidRequest("missing Codex chat history".into());

    assert!(should_skip_codex_history_after_error(&candidate("openai:cli", "openai:chat"), &error));
    assert!(!should_skip_codex_history_after_error(&candidate("openai:cli", "openai:cli"), &error));
    assert!(!should_skip_codex_history_after_error(&candidate("openai:cli", "openai:chat"), &invalid));
}

fn candidate(client_api_format: &str, provider_api_format: &str) -> ProxyCandidate {
    ProxyCandidate {
        trace: CandidateTrace {
            token_id: Some("token-1".into()),
            user_id_snapshot: Some("user-1".into()),
            username_snapshot: Some("alice".into()),
            token_name_snapshot: Some("token".into()),
            token_prefix_snapshot: Some("sk-test".into()),
            group_code: Some("default".into()),
            global_model_id: "model-1".into(),
            provider_model_id: "provider-model-1".into(),
            model_name_snapshot: "gpt-5.5".into(),
            provider_id: "provider-1".into(),
            provider_name_snapshot: "Provider".into(),
            endpoint_id: "endpoint-1".into(),
            endpoint_name_snapshot: provider_api_format.into(),
            key_id: "key-1".into(),
            key_name_snapshot: "Key".into(),
            key_preview_snapshot: "***test".into(),
            client_api_format: client_api_format.into(),
            provider_api_format: provider_api_format.into(),
            needs_conversion: client_api_format != provider_api_format,
            is_stream: false,
            is_cached: false,
            routing_profile_id: types::provider::RoutingProfileId::Balanced,
            routing_profile_ema_alpha: types::provider::default_ema_alpha(),
            routing_context_key: "group=default|model=model-1|format=openai:cli|stream=false|size=unknown|cap=none".into(),
            route_config_fingerprint: "route-fingerprint".into(),
            price_config_fingerprint: "price-fingerprint".into(),
            candidate_index: 0,
        },
        requested_model_name: "gpt-5.5".into(),
        api_key: "secret".into(),
        base_url: "https://example.com".into(),
        custom_path: None,
        upstream_url: "https://example.com/v1/chat/completions".into(),
        provider_model_name: "upstream-model".into(),
        reasoning_effort: None,
        header_rules: None,
        body_rules: None,
        format_acceptance_config: None,
        key_supports_image_generation: false,
        price_per_request: None,
        tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        billing_multiplier: Decimal::ONE,
        max_retries: 0,
        request_timeout_seconds: None,
        stream_first_byte_timeout_seconds: None,
        stream_idle_timeout_seconds: None,
        cache_ttl_minutes: 5,
        key_rpm_limit: None,
        is_cached: false,
        route: CandidateRoute { options: Vec::new() },
    }
}
