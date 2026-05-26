use std::time::{Duration, Instant};

use axum::response::Response;
use req::Response as UpstreamResponse;

use super::{
    LlmProxyError, LlmProxyState, affinity,
    attempt_log::{StartedAttemptInput, record_attempt_error, record_rate_limit_rejection, record_send_error, record_started_attempt},
    failure_classification::{FailureDecision, classify_status},
    outbound_request::{UpstreamRequestBody, UpstreamRequestInput, upstream_request},
    request::{AttemptPayload, PreparedProxyRequest, attempt_payload},
    stream_transport, timeout, transport,
};
use crate::llm_proxy::{
    audit::{SKIP_REASON_REQUEST_TERMINATED, record_skipped_candidates},
    candidate::ProxyCandidate,
    rate_limit,
};

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
    let request = match upstream_request(
        &state.http,
        UpstreamRequestInput {
            candidate,
            target_format: payload.target_format,
            body: UpstreamRequestBody::Json(&payload.body),
            current_body: &payload.body,
            original_body: &payload.original_body,
            provider_headers: &prepared.provider_headers,
            is_stream: prepared.is_stream,
        },
    ) {
        Ok(request) => request,
        Err(error) => {
            let outcome = record_attempt_error(state, &prepared.request_id, candidate, retry_index, error, last_error).await?;
            return Ok(option_response_outcome(outcome));
        }
    };
    let attempt_cancel = record_started_attempt(StartedAttemptInput {
        state,
        request_id: &prepared.request_id,
        candidate,
        retry_index,
        started,
        request: &request,
        provider_body: &payload.body,
    })
    .await?;
    let request_timeout = timeout::non_stream_total_timeout(candidate, prepared.is_stream);
    let response_start_timeout = timeout::response_start_timeout(candidate, prepared.is_stream);
    let response = match execute_upstream_request(&state.http, request, response_start_timeout).await {
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
    handle_upstream_response(HandleUpstreamResponseInput {
        state: state.clone(),
        prepared,
        candidate,
        retry_index,
        started,
        payload,
        response,
        request_timeout,
        failures: (last_failure, last_error),
    })
    .await
}

async fn execute_upstream_request(
    http: &req::ReqwestClient,
    request: req::Request,
    response_start_timeout: Option<Duration>,
) -> Result<UpstreamResponse, req::ClientError> {
    let execute = http.execute(request);
    match response_start_timeout {
        Some(timeout) => tokio::time::timeout(timeout, execute).await.unwrap_or(Err(req::ClientError::Timeout)),
        None => execute.await,
    }
}

struct HandleUpstreamResponseInput<'a> {
    state: LlmProxyState,
    prepared: &'a PreparedProxyRequest,
    candidate: &'a ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    response: UpstreamResponse,
    request_timeout: Option<Duration>,
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
        request_timeout: input.request_timeout,
    })
    .await
    {
        Ok(response) => {
            affinity::remember(&input.state, input.candidate, input.prepared.cache_affinity_ttl_minutes).await?;
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
    affinity::invalidate_retryable(state, candidate, decision).await?;
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
    request_timeout: Option<Duration>,
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
            provider_request_body: input.payload.body,
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
        request_timeout: input.request_timeout,
    })
    .await
}
