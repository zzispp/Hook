use std::time::{Duration, Instant};

use axum::{http::HeaderMap, response::Response};
use proxy::format_conversion::ApiFormat;
use types::api_token::ApiToken;

use super::{
    LlmProxyError, LlmProxyState, affinity,
    attempt_log::{StartedAttemptInput, record_attempt_error, record_rate_limit_rejection, record_send_error, record_started_attempt},
    capture::RequestCapture,
    failure_classification::{FailureDecision, classify_status},
    image_form::MultipartImageRequest,
    outbound_request::{UpstreamRequestBody, UpstreamRequestInput, upstream_request},
    timeout, transport,
};
use crate::llm_proxy::{
    CurrentApiToken, OPENAI_IMAGE_EDIT_FORMAT,
    audit::{SKIP_REASON_REQUEST_TERMINATED, record_scheduled_candidates, record_skipped_candidates},
    billing::enforce_preflight_access,
    candidate::{CandidateRequest, ProxyCandidate, select_candidates},
    rate_limit,
};

pub(super) async fn execute_image_edit_request(
    state: LlmProxyState,
    token: CurrentApiToken,
    headers: HeaderMap,
    request: MultipartImageRequest,
) -> Result<Response, LlmProxyError> {
    let capture = RequestCapture::new(&headers, request.record_body());
    let prepared = prepare_image_edit_request(&state, &token.0, request, capture).await?;
    execute_prepared_image_edit(state, prepared).await
}

struct PreparedImageEditRequest {
    request_id: String,
    cache_affinity_ttl_minutes: i64,
    candidates: Vec<ProxyCandidate>,
    request: MultipartImageRequest,
}

async fn prepare_image_edit_request(
    state: &LlmProxyState,
    token: &ApiToken,
    request: MultipartImageRequest,
    capture: RequestCapture,
) -> Result<PreparedImageEditRequest, LlmProxyError> {
    enforce_preflight_access(state, token).await?;
    rate_limit::enforce_request_limits(state, token).await?;
    let selection = select_candidates(
        state,
        token,
        CandidateRequest {
            api_format: OPENAI_IMAGE_EDIT_FORMAT,
            model_name: request.model(),
            is_stream: false,
        },
    )
    .await?;
    record_scheduled_candidates(state, &selection, &capture).await?;
    Ok(PreparedImageEditRequest {
        request_id: selection.request_id,
        cache_affinity_ttl_minutes: selection.cache_affinity_ttl_minutes,
        candidates: selection.candidates,
        request,
    })
}

async fn execute_prepared_image_edit(state: LlmProxyState, prepared: PreparedImageEditRequest) -> Result<Response, LlmProxyError> {
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
    prepared: &PreparedImageEditRequest,
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
    prepared: &PreparedImageEditRequest,
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
    let provider_body = prepared.request.provider_body(&candidate.provider_model_name);
    let request = match build_upstream_request(state, prepared, candidate, &provider_body) {
        Ok(request) => request,
        Err(error) => {
            let outcome = record_attempt_error(state, &prepared.request_id, candidate, retry_index, error, last_error).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    let started = Instant::now();
    let attempt_cancel = record_started_attempt(StartedAttemptInput {
        state,
        request_id: &prepared.request_id,
        candidate,
        retry_index,
        started,
        request: &request,
        provider_body: &provider_body,
    })
    .await?;
    let request_timeout = timeout::non_stream_total_timeout(candidate, false);
    let response = match execute_upstream_request(&state.http, request, request_timeout).await {
        Ok(response) => {
            attempt_cancel.disarm();
            response
        }
        Err(error) => {
            attempt_cancel.disarm();
            let outcome = record_send_error(state, &prepared.request_id, candidate, retry_index, started, &error, last_error).await?;
            affinity::invalidate_matching(state, candidate).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    handle_response(
        HandleResponseInput {
            state,
            prepared,
            candidate,
            retry_index,
            started,
            request_timeout,
            last_failure,
        },
        response,
    )
    .await
}

fn build_upstream_request(
    state: &LlmProxyState,
    prepared: &PreparedImageEditRequest,
    candidate: &ProxyCandidate,
    provider_body: &serde_json::Value,
) -> Result<req::Request, LlmProxyError> {
    let provider_headers = HeaderMap::new();
    upstream_request(
        &state.http,
        UpstreamRequestInput {
            candidate,
            target_format: ApiFormat::OpenAiImage,
            body: UpstreamRequestBody::Multipart(prepared.request.build_form(&candidate.provider_model_name)?),
            current_body: provider_body,
            original_body: prepared.request.record_body(),
            provider_headers: &provider_headers,
            is_stream: false,
        },
    )
}

async fn execute_upstream_request(
    http: &req::ReqwestClient,
    request: req::Request,
    request_timeout: Option<Duration>,
) -> Result<req::Response, req::ClientError> {
    let execute = http.execute(request);
    match request_timeout {
        Some(timeout) => tokio::time::timeout(timeout, execute).await.unwrap_or(Err(req::ClientError::Timeout)),
        None => execute.await,
    }
}

struct HandleResponseInput<'a> {
    state: &'a LlmProxyState,
    prepared: &'a PreparedImageEditRequest,
    candidate: &'a ProxyCandidate,
    retry_index: i32,
    started: Instant,
    request_timeout: Option<Duration>,
    last_failure: &'a mut Option<transport::UpstreamFailure>,
}

async fn handle_response(input: HandleResponseInput<'_>, response: req::Response) -> Result<AttemptOnceOutcome, LlmProxyError> {
    if !response.status().is_success() {
        return handle_upstream_failure(
            input.state,
            input.prepared,
            input.candidate,
            input.retry_index,
            input.started,
            response,
            input.last_failure,
        )
        .await;
    }
    let response = transport::full_response(transport::FullResponseArgs {
        state: input.state.clone(),
        request_id: input.prepared.request_id.clone(),
        response,
        candidate: input.candidate.clone(),
        service_tier: None,
        source_format: ApiFormat::OpenAiImage,
        target_format: ApiFormat::OpenAiImage,
        started: input.started,
        retry_index: input.retry_index,
        request_timeout: input.request_timeout,
    })
    .await?;
    affinity::remember(input.state, input.candidate, input.prepared.cache_affinity_ttl_minutes).await?;
    Ok(AttemptOnceOutcome::Response(response))
}

async fn handle_upstream_failure(
    state: &LlmProxyState,
    prepared: &PreparedImageEditRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    response: req::Response,
    last_failure: &mut Option<transport::UpstreamFailure>,
) -> Result<AttemptOnceOutcome, LlmProxyError> {
    let decision = classify_status(response.status());
    let failure = transport::record_upstream_failure(
        state,
        &prepared.request_id,
        response,
        candidate,
        started,
        retry_index,
        decision.records_provider_cooldown(),
    )
    .await?;
    affinity::invalidate_retryable(state, candidate, decision).await?;
    classify_failure_outcome(decision, failure, last_failure)
}

fn classify_failure_outcome(
    decision: FailureDecision,
    failure: transport::UpstreamFailure,
    last_failure: &mut Option<transport::UpstreamFailure>,
) -> Result<AttemptOnceOutcome, LlmProxyError> {
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
