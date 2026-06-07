use std::time::{Duration, Instant};

use axum::response::Response;
use req::Response as UpstreamResponse;

use super::{
    LlmProxyError, LlmProxyState, affinity,
    attempt_log::{
        AttemptCancelGuard, AttemptCancelHandle, StartedAttemptInput, record_attempt_error, record_probe_slot_timeout, record_rate_limit_rejection,
        record_send_error, record_started_attempt, record_stream_candidate_watchdog_timeout,
    },
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
    FullResponse(Box<transport::FullResponseArgs>, Box<AttemptCancelGuard>),
}

enum AttemptOnceOutcome {
    ContinueCandidate,
    NextCandidate,
    Response(Response),
    FullResponse(Box<transport::FullResponseArgs>, Box<AttemptCancelGuard>),
}

pub(super) async fn execute_proxy_request(state: LlmProxyState, prepared: PreparedProxyRequest) -> Result<Response, LlmProxyError> {
    let mut last_failure = None;
    let mut last_error = None;
    for candidate in &prepared.candidates {
        let outcome = attempt_candidate(&state, &prepared, candidate, &mut last_failure, &mut last_error).await?;
        match outcome {
            AttemptCandidateOutcome::Continue => {}
            AttemptCandidateOutcome::Response(response) => {
                record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
                return Ok(response);
            }
            AttemptCandidateOutcome::FullResponse(args, attempt_cancel) => {
                let response = transport::full_response(*args).await;
                attempt_cancel.disarm();
                let response = response?;
                affinity::remember(&state, candidate, prepared.cache_affinity_ttl_minutes).await?;
                record_skipped_candidates(&state, &prepared.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
                return Ok(response);
            }
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
            AttemptOnceOutcome::FullResponse(args, attempt_cancel) => return Ok(AttemptCandidateOutcome::FullResponse(args, attempt_cancel)),
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
    if let Some(options) = prepared.provider_key_probe_slot_options {
        match rate_limit::claim_provider_key_probe_slot(state, &candidate.trace.key_id, options).await? {
            rate_limit::ProviderKeyProbeSlotClaim::Claimed => {}
            rate_limit::ProviderKeyProbeSlotClaim::TimedOut(error) => {
                record_probe_slot_timeout(state, &prepared.request_id, candidate, retry_index, error, last_error).await?;
                return Ok(probe_slot_timeout_outcome());
            }
        }
    }
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
    if prepared.is_stream {
        return attempt_stream_candidate_with_watchdog(
            state,
            prepared,
            candidate,
            retry_index,
            started,
            payload,
            request,
            attempt_cancel,
            last_failure,
            last_error,
        )
        .await;
    }
    let request_timeout = timeout::non_stream_total_timeout(candidate, prepared.is_stream);
    let response_start_timeout = timeout::response_start_timeout(candidate, prepared.is_stream);
    let response = match execute_upstream_request(&state.http, request, response_start_timeout).await {
        Ok(response) => {
            attempt_cancel.mark_response_started();
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
        attempt_cancel,
        failures: (last_failure, last_error),
    })
    .await
}

struct StreamAttemptTaskOutput {
    outcome: AttemptOnceOutcome,
    last_failure: Option<transport::UpstreamFailure>,
    last_error: Option<LlmProxyError>,
}

enum StreamWatchdogOutcome<T> {
    Completed(T),
    TimedOut,
}

#[allow(clippy::too_many_arguments)]
async fn attempt_stream_candidate_with_watchdog(
    state: &LlmProxyState,
    prepared: &PreparedProxyRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    request: req::Request,
    attempt_cancel: AttemptCancelGuard,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<AttemptOnceOutcome, LlmProxyError> {
    let timeout_state = state.clone();
    let timeout_request_id = prepared.request_id.clone();
    let timeout_candidate = candidate.clone();
    let timeout_duration = timeout::stream_candidate_watchdog_timeout(candidate);
    let watchdog_handle = attempt_cancel.handle();
    let output = match run_stream_candidate_watchdog(
        timeout_duration,
        watchdog_handle,
        execute_stream_candidate_task(StreamAttemptTaskInput {
            state: state.clone(),
            request_id: prepared.request_id.clone(),
            cache_affinity_ttl_minutes: prepared.cache_affinity_ttl_minutes,
            candidate: candidate.clone(),
            retry_index,
            started,
            payload,
            request,
            attempt_cancel,
        }),
    )
    .await?
    {
        StreamWatchdogOutcome::Completed(output) => output,
        StreamWatchdogOutcome::TimedOut => {
            record_stream_candidate_watchdog_timeout(&timeout_state, &timeout_request_id, &timeout_candidate, retry_index, started).await?;
            affinity::invalidate_matching(&timeout_state, &timeout_candidate).await?;
            stream_candidate_watchdog_timeout_output()
        }
    };
    *last_failure = output.last_failure;
    *last_error = output.last_error;
    Ok(output.outcome)
}

struct StreamAttemptTaskInput {
    state: LlmProxyState,
    request_id: String,
    cache_affinity_ttl_minutes: i64,
    candidate: ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    request: req::Request,
    attempt_cancel: AttemptCancelGuard,
}

async fn execute_stream_candidate_task(input: StreamAttemptTaskInput) -> Result<StreamAttemptTaskOutput, LlmProxyError> {
    let response_start_timeout = timeout::response_start_timeout(&input.candidate, true);
    let response = match execute_upstream_request(&input.state.http, input.request, response_start_timeout).await {
        Ok(response) => {
            input.attempt_cancel.mark_response_started();
            response
        }
        Err(error) => {
            input.attempt_cancel.disarm();
            let mut last_error = None;
            let outcome = record_send_error(
                &input.state,
                &input.request_id,
                &input.candidate,
                input.retry_index,
                input.started,
                &error,
                &mut last_error,
            )
            .await?;
            affinity::invalidate_matching(&input.state, &input.candidate).await?;
            return Ok(StreamAttemptTaskOutput {
                outcome: option_response_outcome(outcome),
                last_failure: None,
                last_error,
            });
        }
    };
    if !response.status().is_success() {
        let mut last_failure = None;
        let outcome = handle_upstream_failure(
            &input.state,
            &input.request_id,
            &input.candidate,
            input.retry_index,
            input.started,
            response,
            &mut last_failure,
        )
        .await;
        input.attempt_cancel.disarm();
        let outcome = outcome?;
        return Ok(StreamAttemptTaskOutput {
            outcome,
            last_failure,
            last_error: None,
        });
    }
    let response = stream_transport::stream_response(
        stream_transport::StreamResponseArgs {
            state: input.state.clone(),
            request_id: input.request_id.clone(),
            response,
            candidate: input.candidate.clone(),
            source_format: input.payload.source_format,
            target_format: input.payload.target_format,
            provider_request_body: input.payload.body,
            started: input.started,
            retry_index: input.retry_index,
        },
        &input.attempt_cancel,
    )
    .await;
    match response {
        Ok(response) => {
            affinity::remember(&input.state, &input.candidate, input.cache_affinity_ttl_minutes).await?;
            Ok(StreamAttemptTaskOutput {
                outcome: AttemptOnceOutcome::Response(response),
                last_failure: None,
                last_error: None,
            })
        }
        Err(error) => Ok(StreamAttemptTaskOutput {
            outcome: AttemptOnceOutcome::ContinueCandidate,
            last_failure: None,
            last_error: Some(error),
        }),
    }
}

async fn run_stream_candidate_watchdog<T>(
    timeout_duration: Option<Duration>,
    watchdog_handle: AttemptCancelHandle,
    future: impl std::future::Future<Output = Result<T, LlmProxyError>> + Send + 'static,
) -> Result<StreamWatchdogOutcome<T>, LlmProxyError>
where
    T: Send + 'static,
{
    let Some(timeout_duration) = timeout_duration else {
        return future.await.map(StreamWatchdogOutcome::Completed);
    };
    let mut join_handle = tokio::spawn(future);
    match tokio::time::timeout(timeout_duration, &mut join_handle).await {
        Ok(Ok(result)) => result.map(StreamWatchdogOutcome::Completed),
        Ok(Err(error)) => Err(LlmProxyError::Infrastructure(format!("stream candidate task join failed: {error}"))),
        Err(_) => {
            watchdog_handle.disarm();
            join_handle.abort();
            Ok(StreamWatchdogOutcome::TimedOut)
        }
    }
}

fn stream_candidate_watchdog_timeout_output() -> StreamAttemptTaskOutput {
    StreamAttemptTaskOutput {
        outcome: AttemptOnceOutcome::NextCandidate,
        last_failure: Some(transport::gateway_timeout_failure()),
        last_error: Some(LlmProxyError::Upstream("stream candidate watchdog timed out".into())),
    }
}

fn probe_slot_timeout_outcome() -> AttemptOnceOutcome {
    AttemptOnceOutcome::ContinueCandidate
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
    attempt_cancel: AttemptCancelGuard,
    failures: (&'a mut Option<transport::UpstreamFailure>, &'a mut Option<LlmProxyError>),
}

async fn handle_upstream_response(input: HandleUpstreamResponseInput<'_>) -> Result<AttemptOnceOutcome, LlmProxyError> {
    if !input.response.status().is_success() {
        let outcome = handle_upstream_failure(
            &input.state,
            &input.prepared.request_id,
            input.candidate,
            input.retry_index,
            input.started,
            input.response,
            input.failures.0,
        )
        .await;
        input.attempt_cancel.disarm();
        let outcome = outcome?;
        return Ok(outcome);
    }
    let outcome = success_response(SuccessResponseInput {
        state: input.state.clone(),
        prepared: input.prepared,
        candidate: input.candidate,
        retry_index: input.retry_index,
        started: input.started,
        payload: input.payload,
        response: input.response,
        request_timeout: input.request_timeout,
        attempt_cancel: input.attempt_cancel,
    })
    .await;
    match outcome {
        Ok(AttemptOnceOutcome::Response(response)) => {
            affinity::remember(&input.state, input.candidate, input.prepared.cache_affinity_ttl_minutes).await?;
            Ok(AttemptOnceOutcome::Response(response))
        }
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            *input.failures.1 = Some(error);
            Ok(AttemptOnceOutcome::ContinueCandidate)
        }
    }
}

async fn handle_upstream_failure(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    response: UpstreamResponse,
    last_failure: &mut Option<transport::UpstreamFailure>,
) -> Result<AttemptOnceOutcome, LlmProxyError> {
    let decision = classify_status(response.status());
    let record_cooldown = decision.records_provider_cooldown();
    let failure = transport::record_upstream_failure(state, request_id, response, candidate, started, retry_index, record_cooldown).await?;
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
    attempt_cancel: AttemptCancelGuard,
}

async fn success_response(input: SuccessResponseInput<'_>) -> Result<AttemptOnceOutcome, LlmProxyError> {
    if input.prepared.is_stream {
        return stream_transport::stream_response(
            stream_transport::StreamResponseArgs {
                state: input.state,
                request_id: input.prepared.request_id.clone(),
                response: input.response,
                candidate: input.candidate.clone(),
                source_format: input.payload.source_format,
                target_format: input.payload.target_format,
                provider_request_body: input.payload.body,
                started: input.started,
                retry_index: input.retry_index,
            },
            &input.attempt_cancel,
        )
        .await
        .map(AttemptOnceOutcome::Response);
    }
    let response = transport::FullResponseArgs {
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
    };
    Ok(AttemptOnceOutcome::FullResponse(Box::new(response), Box::new(input.attempt_cancel)))
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod tests;
