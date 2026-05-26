use std::time::Instant;

use axum::response::Response;
use types::model::PatchField;

use super::{LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt},
    candidate::ProxyCandidate,
};

pub(super) struct AttemptCancelGuard {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    retry_index: i32,
    started: Instant,
    armed: bool,
}

impl AttemptCancelGuard {
    pub(super) fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for AttemptCancelGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let input = CancelledAttemptInput {
            state: self.state.clone(),
            request_id: self.request_id.clone(),
            candidate: self.candidate.clone(),
            retry_index: self.retry_index,
            latency_ms: elapsed_ms(self.started),
        };
        tokio::spawn(async move {
            if let Err(error) = record_cancelled_attempt(input).await {
                hook_tracing::warn_with_fields!("failed to record cancelled provider attempt", error = error);
            }
        });
    }
}

struct CancelledAttemptInput {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    retry_index: i32,
    latency_ms: i64,
}

pub(super) struct StartedAttemptInput<'a> {
    pub(super) state: &'a LlmProxyState,
    pub(super) request_id: &'a str,
    pub(super) candidate: &'a ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) started: Instant,
    pub(super) request: &'a req::Request,
    pub(super) provider_body: &'a serde_json::Value,
}

pub(super) async fn record_attempt_error(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error: LlmProxyError,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    record_failed_attempt(state, request_id, candidate, retry_index, "request_conversion_error", &error).await?;
    *last_error = Some(error);
    Ok(None)
}

pub(super) async fn record_rate_limit_rejection(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error: LlmProxyError,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    record_failed_attempt(state, request_id, candidate, retry_index, "provider_key_rate_limit", &error).await?;
    *last_error = Some(error);
    Ok(None)
}

async fn record_failed_attempt(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error_type: &'static str,
    error: &LlmProxyError,
) -> Result<(), LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            error_type: Some(error_type),
            error_message: Some(error_message.as_str()),
            ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
        },
    )
    .await
}

pub(super) async fn record_started_attempt(input: StartedAttemptInput<'_>) -> Result<AttemptCancelGuard, LlmProxyError> {
    record_attempt(
        input.state,
        input.request_id,
        AttemptRecordInput {
            status: "pending",
            provider_request_headers: PatchField::Value(input.request.headers().clone()),
            provider_request_body: PatchField::Value(input.provider_body.clone()),
            client_response_headers: PatchField::Null,
            client_response_body: PatchField::Null,
            ..AttemptRecordInput::new(input.candidate, input.retry_index, "pending", false)
        },
    )
    .await?;
    Ok(AttemptCancelGuard {
        state: input.state.clone(),
        request_id: input.request_id.to_owned(),
        candidate: input.candidate.clone(),
        retry_index: input.retry_index,
        started: input.started,
        armed: true,
    })
}

pub(super) async fn record_send_error(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    error: &req::ClientError,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        request_id,
        AttemptRecordInput {
            latency_ms: Some(elapsed_ms(started)),
            error_type: Some(send_error_type(error)),
            error_message: Some(error_message.as_str()),
            ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
        },
    )
    .await?;
    *last_error = Some(LlmProxyError::Upstream(error_message));
    Ok(None)
}

fn send_error_type(error: &req::ClientError) -> &'static str {
    if matches!(error, req::ClientError::Timeout) {
        return "upstream_timeout";
    }
    "upstream_send_error"
}

async fn record_cancelled_attempt(input: CancelledAttemptInput) -> Result<(), LlmProxyError> {
    record_attempt(
        &input.state,
        &input.request_id,
        cancelled_attempt_record(&input.candidate, input.retry_index, input.latency_ms),
    )
    .await
}

fn cancelled_attempt_record(candidate: &ProxyCandidate, retry_index: i32, latency_ms: i64) -> AttemptRecordInput<'_> {
    AttemptRecordInput {
        status_code: Some(499),
        latency_ms: Some(latency_ms),
        error_type: Some("client_disconnected"),
        error_message: Some("client disconnected before upstream response started"),
        termination_origin: PatchField::Value("client".into()),
        termination_reason: PatchField::Value("disconnected".into()),
        stream_end_reason: PatchField::Value("client_gone".into()),
        ..AttemptRecordInput::new(candidate, retry_index, "cancelled", true)
    }
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::{model::PatchField, model::TieredPricingConfig};

    use super::cancelled_attempt_record;
    use crate::llm_proxy::{
        audit::request_billing_status,
        candidate::{CandidateRoute, CandidateTrace, ProxyCandidate},
    };

    #[test]
    fn cancelled_before_upstream_response_is_explicit_terminal_record() {
        let candidate = candidate();
        let input = cancelled_attempt_record(&candidate, 0, 42);

        assert_eq!(input.status, "cancelled");
        assert!(input.finished);
        assert_eq!(input.status_code, Some(499));
        assert_eq!(input.latency_ms, Some(42));
        assert_eq!(input.error_type, Some("client_disconnected"));
        assert_eq!(input.error_message, Some("client disconnected before upstream response started"));
        assert_eq!(input.termination_origin, PatchField::Value("client".into()));
        assert_eq!(input.termination_reason, PatchField::Value("disconnected".into()));
        assert_eq!(input.stream_end_reason, PatchField::Value("client_gone".into()));
        assert_eq!(request_billing_status(&input, None), "void");
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
}
