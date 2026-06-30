use std::time::Instant;

use axum::{http::HeaderMap, response::Response};
use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use super::{
    LlmProxyError, LlmProxyState, affinity,
    attempt_log::{StartedAttemptInput, record_attempt_error, record_rate_limit_rejection, record_send_error, record_started_attempt},
    image_attempt_response::{HandleResponseInput, ImageHandleResponseOutcome, handle_response},
    image_prepared::{PreparedImageRequest, UpstreamImageBody},
    image_stream_mode::candidate_image_stream_mode,
    image_stream_wrapper::{StreamImageRequest, failure_sse_body, response_to_bytes},
    outbound_request::{UpstreamRequestBody, UpstreamRequestInput, upstream_request},
    timeout, transport,
};
use crate::llm_proxy::{
    audit::{SKIP_REASON_REQUEST_TERMINATED, record_skipped_candidates},
    candidate::ProxyCandidate,
    rate_limit,
};

pub(super) async fn execute_sync_client_response(state: LlmProxyState, prepared: PreparedImageRequest) -> Result<Response, LlmProxyError> {
    let mut last_failure = None;
    let mut last_error = None;
    for candidate in &prepared.candidates {
        if let AttemptCandidateOutcome::Response(response) = attempt_candidate(&state, &prepared, candidate, &mut last_failure, &mut last_error).await? {
            record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
            return Ok(response);
        }
    }
    record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
    if let Some(failure) = last_failure {
        return transport::failure_response(failure);
    }
    Err(last_error.unwrap_or_else(|| LlmProxyError::Upstream("all provider candidates failed".into())))
}

pub(super) fn spawn_stream_image_attempts(state: LlmProxyState, request: StreamImageRequest) -> tokio::task::JoinHandle<Result<Vec<u8>, LlmProxyError>> {
    tokio::spawn(async move { execute_stream_image_attempts(state, request.into()).await })
}

async fn execute_stream_image_attempts(state: LlmProxyState, prepared: PreparedImageRequest) -> Result<Vec<u8>, LlmProxyError> {
    let mut last_failure = None;
    let mut last_error = None;
    for candidate in &prepared.candidates {
        match attempt_candidate(&state, &prepared, candidate, &mut last_failure, &mut last_error).await? {
            AttemptCandidateOutcome::Response(response) => {
                record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
                return response_to_bytes(response).await;
            }
            AttemptCandidateOutcome::Continue => {}
        }
    }
    record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
    if let Some(failure) = last_failure {
        return failure_sse_body(failure);
    }
    Err(last_error.unwrap_or_else(|| LlmProxyError::Upstream("all provider candidates failed".into())))
}

enum AttemptCandidateOutcome {
    Continue,
    Response(Response),
}

enum AttemptOnceOutcome {
    ContinueCandidate,
    NextCandidate,
    Response(Response),
}

async fn attempt_candidate(
    state: &LlmProxyState,
    prepared: &PreparedImageRequest,
    candidate: &ProxyCandidate,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<AttemptCandidateOutcome, LlmProxyError> {
    for retry_index in affinity::attempt_range(candidate) {
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
    prepared: &PreparedImageRequest,
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
    let upstream_is_stream = candidate_image_stream_mode(candidate)?.upstream_is_stream();
    let provider_body = prepared.body.provider_body(candidate, upstream_is_stream)?;
    let request = match build_upstream_request(state, prepared, candidate, upstream_is_stream, &provider_body) {
        Ok(request) => request,
        Err(error) => {
            let outcome = record_attempt_error(state, &prepared.request_id, candidate, retry_index, error, last_error).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    execute_attempt_request(ExecuteAttemptInput {
        state,
        prepared,
        candidate,
        retry_index,
        upstream_is_stream,
        provider_body,
        request,
        last_failure,
        last_error,
    })
    .await
}

struct ExecuteAttemptInput<'a> {
    state: &'a LlmProxyState,
    prepared: &'a PreparedImageRequest,
    candidate: &'a ProxyCandidate,
    retry_index: i32,
    upstream_is_stream: bool,
    provider_body: Value,
    request: req::Request,
    last_failure: &'a mut Option<transport::UpstreamFailure>,
    last_error: &'a mut Option<LlmProxyError>,
}

async fn execute_attempt_request(input: ExecuteAttemptInput<'_>) -> Result<AttemptOnceOutcome, LlmProxyError> {
    let started = Instant::now();
    let attempt_cancel = record_started_attempt(StartedAttemptInput {
        state: input.state,
        request_id: &input.prepared.request_id,
        candidate: input.candidate,
        retry_index: input.retry_index,
        started,
        request: &input.request,
        provider_body: &input.provider_body,
    })
    .await?;
    let response = match execute_upstream_request(&input.state.http, input.request, input.upstream_is_stream, input.candidate).await {
        Ok(response) => {
            attempt_cancel.mark_response_started();
            response
        }
        Err(error) => {
            attempt_cancel.disarm();
            let outcome = record_send_error(
                input.state,
                &input.prepared.request_id,
                input.candidate,
                input.retry_index,
                started,
                timeout_error_type(input.upstream_is_stream),
                &error,
                input.last_error,
            )
            .await?;
            affinity::invalidate_matching(input.state, input.candidate).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    let response = handle_response_input(HandleResponseInput {
        state: input.state,
        prepared: input.prepared,
        candidate: input.candidate,
        retry_index: input.retry_index,
        started,
        upstream_is_stream: input.upstream_is_stream,
        response,
        attempt_cancel: &attempt_cancel,
        last_failure: input.last_failure,
    })
    .await;
    attempt_cancel.disarm();
    response
}

async fn handle_response_input(input: HandleResponseInput<'_>) -> Result<AttemptOnceOutcome, LlmProxyError> {
    match handle_response(input).await? {
        ImageHandleResponseOutcome::ContinueCandidate => Ok(AttemptOnceOutcome::ContinueCandidate),
        ImageHandleResponseOutcome::NextCandidate => Ok(AttemptOnceOutcome::NextCandidate),
        ImageHandleResponseOutcome::Response(response) => Ok(AttemptOnceOutcome::Response(response)),
    }
}

fn option_response_outcome(response: Option<Response>) -> AttemptOnceOutcome {
    match response {
        Some(response) => AttemptOnceOutcome::Response(response),
        None => AttemptOnceOutcome::ContinueCandidate,
    }
}

fn build_upstream_request(
    state: &LlmProxyState,
    prepared: &PreparedImageRequest,
    candidate: &ProxyCandidate,
    upstream_is_stream: bool,
    provider_body: &Value,
) -> Result<req::Request, LlmProxyError> {
    let provider_headers = HeaderMap::new();
    let body = prepared.body.upstream_body(candidate, upstream_is_stream)?;
    match body {
        UpstreamImageBody::Json(value) => upstream_request(
            &state.http,
            UpstreamRequestInput {
                candidate,
                target_format: ApiFormat::OpenAiImage,
                body: UpstreamRequestBody::Json(&value),
                current_body: provider_body,
                original_body: prepared.body.original_body(),
                provider_headers: &provider_headers,
                is_stream: upstream_is_stream,
            },
        ),
        UpstreamImageBody::Multipart(form) => upstream_request(
            &state.http,
            UpstreamRequestInput {
                candidate,
                target_format: ApiFormat::OpenAiImage,
                body: UpstreamRequestBody::Multipart(form),
                current_body: provider_body,
                original_body: prepared.body.original_body(),
                provider_headers: &provider_headers,
                is_stream: upstream_is_stream,
            },
        ),
    }
}

fn timeout_error_type(is_stream: bool) -> &'static str {
    if is_stream {
        return "response_headers_timeout";
    }
    "upstream_timeout"
}

async fn execute_upstream_request(
    http: &req::ReqwestClient,
    request: req::Request,
    upstream_is_stream: bool,
    candidate: &ProxyCandidate,
) -> Result<req::Response, req::ClientError> {
    let timeout = timeout::upstream_response_headers_timeout(candidate, upstream_is_stream);
    let execute = http.execute(request);
    match timeout {
        Some(timeout) => tokio::time::timeout(timeout, execute).await.unwrap_or(Err(req::ClientError::Timeout)),
        None => execute.await,
    }
}
