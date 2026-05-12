use std::time::{Duration, Instant};

use axum::response::Response;
use proxy::format_conversion::ApiFormat;
use reqwest::StatusCode;
use serde_json::Value;

use super::{
    LlmProxyError, LlmProxyState,
    attempt_log::{record_attempt_error, record_send_error, record_started_attempt},
    request::{AttemptPayload, PreparedProxyRequest, attempt_payload},
    stream_transport, transport,
};
use crate::llm_proxy::candidate::ProxyCandidate;

const ANTHROPIC_VERSION: &str = "2023-06-01";
const RETRYABLE_AUTH_STATUS_UNAUTHORIZED: StatusCode = StatusCode::UNAUTHORIZED;
const RETRYABLE_AUTH_STATUS_FORBIDDEN: StatusCode = StatusCode::FORBIDDEN;
const RETRYABLE_STATUS_TIMEOUT: StatusCode = StatusCode::REQUEST_TIMEOUT;
const RETRYABLE_STATUS_RATE_LIMIT: StatusCode = StatusCode::TOO_MANY_REQUESTS;

pub(super) async fn execute_proxy_request(state: LlmProxyState, prepared: PreparedProxyRequest) -> Result<Response, LlmProxyError> {
    let mut last_failure = None;
    let mut last_error = None;
    for candidate in &prepared.candidates {
        let outcome = attempt_candidate(&state, &prepared, candidate, &mut last_failure, &mut last_error).await?;
        if let Some(response) = outcome {
            return Ok(response);
        }
    }
    if let Some(failure) = last_failure {
        return transport::failure_response(failure);
    }
    Err(last_error.unwrap_or_else(|| LlmProxyError::Upstream("all provider candidates failed".into())))
}

async fn attempt_candidate(
    state: &LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    for retry_index in 0..=candidate.max_retries {
        let Some(response) = attempt_once(state, prepared, candidate, retry_index, last_failure, last_error).await? else {
            continue;
        };
        return Ok(Some(response));
    }
    Ok(None)
}

async fn attempt_once(
    state: &LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    let payload = match attempt_payload(prepared.body.clone(), candidate, prepared.force_non_stream) {
        Ok(payload) => payload,
        Err(error) => return record_attempt_error(state, &prepared.request_id, candidate, retry_index, error, last_error).await,
    };
    record_started_attempt(state, &prepared.request_id, candidate, prepared.is_stream, retry_index).await?;
    let started = Instant::now();
    let response = match upstream_request(&state.http, candidate, payload.target_format, &payload.body).send().await {
        Ok(response) => response,
        Err(error) => return record_send_error(state, &prepared.request_id, candidate, retry_index, started, &error, last_error).await,
    };
    handle_upstream_response(
        state.clone(),
        prepared,
        candidate,
        retry_index,
        started,
        payload,
        response,
        (last_failure, last_error),
    )
    .await
}

async fn handle_upstream_response(
    state: LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    response: reqwest::Response,
    failures: (&mut Option<transport::UpstreamFailure>, &mut Option<LlmProxyError>),
) -> Result<Option<Response>, LlmProxyError> {
    if !response.status().is_success() {
        return handle_upstream_failure(&state, prepared, candidate, retry_index, started, response, failures.0).await;
    }
    match success_response(state.clone(), prepared, candidate, retry_index, started, payload, response).await {
        Ok(response) => {
            remember_affinity(&state, candidate).await?;
            Ok(Some(response))
        }
        Err(error) => {
            *failures.1 = Some(error);
            Ok(None)
        }
    }
}

async fn handle_upstream_failure(
    state: &LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    response: reqwest::Response,
    last_failure: &mut Option<transport::UpstreamFailure>,
) -> Result<Option<Response>, LlmProxyError> {
    let retryable = status_retryable(response.status());
    let failure = transport::record_upstream_failure(state, &prepared.request_id, response, candidate, started, retry_index).await?;
    if retryable {
        *last_failure = Some(failure);
        return Ok(None);
    }
    transport::failure_response(failure).map(Some)
}

async fn success_response(
    state: LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    response: reqwest::Response,
) -> Result<Response, LlmProxyError> {
    if prepared.is_stream {
        return stream_response(state, prepared.request_id.clone(), response, candidate.clone(), payload, started, retry_index).await;
    }
    full_response(state, prepared.request_id.clone(), response, candidate.clone(), payload, started, retry_index).await
}

fn upstream_request(client: &reqwest::Client, candidate: &ProxyCandidate, target_format: ApiFormat, body: &Value) -> reqwest::RequestBuilder {
    let builder = client.post(candidate.upstream_url.clone()).json(body);
    let builder = if candidate.trace.provider_api_format == "claude_cli" {
        builder.bearer_auth(candidate.api_key.as_str())
    } else {
        match target_format {
            ApiFormat::ClaudeChat => builder
                .header("x-api-key", candidate.api_key.as_str())
                .header("anthropic-version", ANTHROPIC_VERSION),
            ApiFormat::GeminiChat => builder.header("x-goog-api-key", candidate.api_key.as_str()),
            ApiFormat::OpenAiChat | ApiFormat::OpenAiResponses => builder.bearer_auth(candidate.api_key.as_str()),
        }
    };
    apply_timeout(builder, candidate)
}

fn apply_timeout(builder: reqwest::RequestBuilder, candidate: &ProxyCandidate) -> reqwest::RequestBuilder {
    let timeout_seconds = if candidate.trace.is_stream {
        candidate.stream_first_byte_timeout_seconds
    } else {
        candidate.request_timeout_seconds
    };
    match timeout_seconds.and_then(timeout_duration) {
        Some(timeout) => builder.timeout(timeout),
        None => builder,
    }
}

fn timeout_duration(seconds: f64) -> Option<Duration> {
    (seconds.is_finite() && seconds > 0.0).then(|| Duration::from_secs_f64(seconds))
}

fn status_retryable(status: StatusCode) -> bool {
    status.is_server_error()
        || matches!(
            status,
            RETRYABLE_AUTH_STATUS_UNAUTHORIZED | RETRYABLE_AUTH_STATUS_FORBIDDEN | RETRYABLE_STATUS_TIMEOUT | RETRYABLE_STATUS_RATE_LIMIT
        )
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

async fn full_response(
    state: LlmProxyState,
    request_id: String,
    response: reqwest::Response,
    candidate: ProxyCandidate,
    payload: AttemptPayload,
    started: Instant,
    retry_index: i32,
) -> Result<Response, LlmProxyError> {
    transport::full_response(
        state,
        request_id,
        response,
        candidate,
        payload.source_format,
        payload.target_format,
        started,
        retry_index,
    )
    .await
}

async fn stream_response(
    state: LlmProxyState,
    request_id: String,
    response: reqwest::Response,
    candidate: ProxyCandidate,
    payload: AttemptPayload,
    started: Instant,
    retry_index: i32,
) -> Result<Response, LlmProxyError> {
    stream_transport::stream_response(
        state,
        request_id,
        response,
        candidate,
        payload.source_format,
        payload.target_format,
        started,
        retry_index,
    )
    .await
}
