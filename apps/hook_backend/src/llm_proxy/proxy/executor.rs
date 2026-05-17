use std::time::Instant;

use axum::response::Response;
use proxy::format_conversion::ApiFormat;
use req::{Request, RequestBuilder, Response as UpstreamResponse};
use serde_json::Value;

use super::{
    LlmProxyError, LlmProxyState,
    attempt_log::{record_attempt_error, record_rate_limit_rejection, record_send_error, record_started_attempt},
    failure_classification::{FailureDecision, classify_status},
    header_rules::apply_provider_header_rules,
    request::{AttemptPayload, PreparedProxyRequest, attempt_payload},
    stream_transport,
    timeout::proxy_timeouts,
    transport,
};
use crate::llm_proxy::{
    audit::{SKIP_REASON_REQUEST_TERMINATED, record_skipped_candidates},
    candidate::ProxyCandidate,
    formats::{self, AuthScheme},
    rate_limit,
};

const ANTHROPIC_VERSION: &str = "2023-06-01";
enum AttemptCandidateOutcome {
    Continue,
    Response(Response),
}

enum AttemptOnceOutcome {
    ContinueCandidate,
    NextCandidate,
    Response(Response),
}

pub(super) async fn execute_proxy_request(state: LlmProxyState, prepared: PreparedProxyRequest) -> Result<Response, LlmProxyError> {
    let mut last_failure = None;
    let mut last_error = None;
    for candidate in &prepared.candidates {
        let outcome = attempt_candidate(&state, &prepared, candidate, &mut last_failure, &mut last_error).await?;
        if let AttemptCandidateOutcome::Response(response) = outcome {
            record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
            return Ok(response);
        }
    }
    if let Some(failure) = last_failure {
        record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
        return transport::failure_response(failure);
    }
    record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
    Err(last_error.unwrap_or_else(|| LlmProxyError::Upstream("all provider candidates failed".into())))
}

async fn attempt_candidate(
    state: &LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<AttemptCandidateOutcome, LlmProxyError> {
    for retry_index in 0..=candidate.max_retries {
        let attempt = candidate.for_attempt(retry_index);
        match attempt_once(state, prepared, &attempt, retry_index, last_failure, last_error).await? {
            AttemptOnceOutcome::ContinueCandidate => {}
            AttemptOnceOutcome::NextCandidate => return Ok(AttemptCandidateOutcome::Continue),
            AttemptOnceOutcome::Response(response) => return Ok(AttemptCandidateOutcome::Response(response)),
        }
    }
    Ok(AttemptCandidateOutcome::Continue)
}

async fn attempt_once(
    state: &LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<AttemptOnceOutcome, LlmProxyError> {
    match rate_limit::claim_provider_key_limit(state, &candidate.trace.key_id, candidate.key_rpm_limit).await {
        Ok(()) => {}
        Err(error @ LlmProxyError::RateLimited(_)) => {
            let outcome = record_rate_limit_rejection(state, &prepared.request_id, candidate, retry_index, error, last_error).await?;
            return Ok(option_response_outcome(outcome));
        }
        Err(error) => return Err(error),
    }
    let payload = match attempt_payload(prepared.body.clone(), candidate, prepared.force_non_stream) {
        Ok(payload) => payload,
        Err(error) => {
            let outcome = record_attempt_error(state, &prepared.request_id, candidate, retry_index, error, last_error).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    let started = Instant::now();
    let request = match upstream_request(&state.http, candidate, payload.target_format, &payload.body, &payload.original_body) {
        Ok(request) => request,
        Err(error) => {
            let outcome = record_attempt_error(state, &prepared.request_id, candidate, retry_index, error, last_error).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    record_started_attempt(state, &prepared.request_id, candidate, prepared.is_stream, retry_index, &request, &payload.body).await?;
    let response = match state.http.execute(request).await {
        Ok(response) => response,
        Err(error) => {
            let outcome = record_send_error(state, &prepared.request_id, candidate, retry_index, started, &error, last_error).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    handle_upstream_response(HandleUpstreamResponseInput {
        state: state.clone(),
        prepared,
        candidate,
        retry_index,
        started,
        payload,
        response,
        failures: (last_failure, last_error),
    })
    .await
}

struct HandleUpstreamResponseInput<'a> {
    state: LlmProxyState,
    prepared: &'a PreparedProxyRequest,
    candidate: &'a ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    response: UpstreamResponse,
    failures: (&'a mut Option<transport::UpstreamFailure>, &'a mut Option<LlmProxyError>),
}

async fn handle_upstream_response(input: HandleUpstreamResponseInput<'_>) -> Result<AttemptOnceOutcome, LlmProxyError> {
    if !input.response.status().is_success() {
        return handle_upstream_failure(
            &input.state,
            input.prepared,
            input.candidate,
            input.retry_index,
            input.started,
            input.response,
            input.failures.0,
        )
        .await;
    }
    match success_response(SuccessResponseInput {
        state: input.state.clone(),
        prepared: input.prepared,
        candidate: input.candidate,
        retry_index: input.retry_index,
        started: input.started,
        payload: input.payload,
        response: input.response,
    })
    .await
    {
        Ok(response) => {
            remember_affinity(&input.state, input.candidate).await?;
            Ok(AttemptOnceOutcome::Response(response))
        }
        Err(error) => {
            *input.failures.1 = Some(error);
            Ok(AttemptOnceOutcome::ContinueCandidate)
        }
    }
}

async fn handle_upstream_failure(
    state: &LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    response: UpstreamResponse,
    last_failure: &mut Option<transport::UpstreamFailure>,
) -> Result<AttemptOnceOutcome, LlmProxyError> {
    let decision = classify_status(response.status());
    let record_cooldown = decision.records_provider_cooldown();
    let failure = transport::record_upstream_failure(state, &prepared.request_id, response, candidate, started, retry_index, record_cooldown).await?;
    match decision {
        FailureDecision::ReturnResponse => transport::failure_response(failure).map(AttemptOnceOutcome::Response),
        FailureDecision::NextCandidate => {
            *last_failure = Some(failure);
            Ok(AttemptOnceOutcome::NextCandidate)
        }
        FailureDecision::RetryOrNextCandidate => {
            let cooldown_triggered = failure.cooldown_triggered();
            *last_failure = Some(failure);
            Ok(if cooldown_triggered {
                AttemptOnceOutcome::NextCandidate
            } else {
                AttemptOnceOutcome::ContinueCandidate
            })
        }
    }
}

fn option_response_outcome(response: Option<Response>) -> AttemptOnceOutcome {
    match response {
        Some(response) => AttemptOnceOutcome::Response(response),
        None => AttemptOnceOutcome::ContinueCandidate,
    }
}

struct SuccessResponseInput<'a> {
    state: LlmProxyState,
    prepared: &'a PreparedProxyRequest,
    candidate: &'a ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    response: UpstreamResponse,
}

async fn success_response(input: SuccessResponseInput<'_>) -> Result<Response, LlmProxyError> {
    if input.prepared.is_stream {
        return stream_transport::stream_response(stream_transport::StreamResponseArgs {
            state: input.state,
            request_id: input.prepared.request_id.clone(),
            response: input.response,
            candidate: input.candidate.clone(),
            source_format: input.payload.source_format,
            target_format: input.payload.target_format,
            started: input.started,
            retry_index: input.retry_index,
        })
        .await;
    }
    transport::full_response(transport::FullResponseArgs {
        state: input.state,
        request_id: input.prepared.request_id.clone(),
        response: input.response,
        candidate: input.candidate.clone(),
        service_tier: input.prepared.service_tier.clone(),
        source_format: input.payload.source_format,
        target_format: input.payload.target_format,
        started: input.started,
        retry_index: input.retry_index,
    })
    .await
}

fn upstream_request(
    client: &req::ReqwestClient,
    candidate: &ProxyCandidate,
    target_format: ApiFormat,
    body: &Value,
    original_body: &Value,
) -> Result<Request, LlmProxyError> {
    let builder = client.post(candidate.upstream_url.clone()).json(body);
    let metadata = formats::endpoint_metadata(
        &candidate.trace.provider_api_format,
        body.get("stream").and_then(Value::as_bool).unwrap_or(false),
    )?;
    if target_format != metadata.data_format {
        return Err(LlmProxyError::InvalidRequest(format!(
            "provider format metadata mismatch: {}",
            candidate.trace.provider_api_format
        )));
    }
    let builder = apply_auth(builder, candidate, metadata.auth_scheme);
    let mut request = client.build_request(apply_timeout(builder, candidate))?;
    apply_provider_header_rules(request.headers_mut(), &candidate.header_rules, body, original_body)?;
    Ok(request)
}

fn apply_auth(builder: RequestBuilder, candidate: &ProxyCandidate, scheme: AuthScheme) -> RequestBuilder {
    match scheme {
        AuthScheme::Bearer => builder.bearer_auth(candidate.api_key.as_str()),
        AuthScheme::Anthropic => builder
            .header("x-api-key", candidate.api_key.as_str())
            .header("anthropic-version", ANTHROPIC_VERSION),
        AuthScheme::Gemini => builder.header("x-goog-api-key", candidate.api_key.as_str()),
    }
}

fn apply_timeout(builder: RequestBuilder, candidate: &ProxyCandidate) -> RequestBuilder {
    match proxy_timeouts(candidate).request {
        Some(timeout) => builder.timeout(timeout),
        None => builder,
    }
}

async fn remember_affinity(state: &LlmProxyState, candidate: &ProxyCandidate) -> Result<(), LlmProxyError> {
    let Some(token_id) = candidate.trace.token_id.as_deref() else {
        return Ok(());
    };
    state
        .remember_affinity_key(
            token_id,
            &candidate.trace.global_model_id,
            &candidate.trace.client_api_format,
            &candidate.trace.key_id,
            candidate.cache_ttl_minutes,
        )
        .await
}
