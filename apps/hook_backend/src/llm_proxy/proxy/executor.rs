use std::time::{Duration, Instant};

use axum::response::Response;
use req::Response as UpstreamResponse;

use super::{
    LlmProxyError, LlmProxyState, affinity,
    attempt_log::{
        AttemptCancelGuard, AttemptCancelHandle, SkippedAttemptInput, StartedAttemptInput, record_attempt_error, record_candidate_skipped_attempt,
        record_probe_slot_timeout, record_rate_limit_rejection, record_send_error, record_skipped_attempt, record_started_attempt,
        record_stream_candidate_watchdog_timeout,
    },
    failure_classification::{FailureDecision, classify_status},
    outbound_request::{UpstreamRequestBody, UpstreamRequestInput, upstream_request},
    request::{AttemptContext, AttemptPayload, PreparedProxyRequest, attempt_payload},
    stream_transport, timeout, transport,
};
use crate::llm_proxy::{
    OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT,
    audit::{SKIP_REASON_REQUEST_TERMINATED, record_skipped_candidates},
    candidate::ProxyCandidate,
    rate_limit,
};

const SKIP_REASON_CODEX_CHAT_HISTORY_UNAVAILABLE: &str = "codex_chat_history_unavailable";
const RESPONSE_HEADERS_TIMEOUT_ERROR_TYPE: &str = "response_headers_timeout";
const UPSTREAM_TIMEOUT_ERROR_TYPE: &str = "upstream_timeout";

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

#[derive(Default)]
struct AttemptExecutionState {
    codex_chat_history_unavailable_message: Option<String>,
}

pub(super) async fn execute_proxy_request(state: LlmProxyState, prepared: PreparedProxyRequest) -> Result<Response, LlmProxyError> {
    let mut last_failure = None;
    let mut last_error = None;
    let mut execution_state = AttemptExecutionState::default();
    for candidate in &prepared.candidates {
        let outcome = attempt_candidate(&state, &prepared, candidate, &mut execution_state, &mut last_failure, &mut last_error).await?;
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
    execution_state: &mut AttemptExecutionState,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<AttemptCandidateOutcome, LlmProxyError> {
    for retry_index in affinity::attempt_range(candidate) {
        let attempt = candidate.for_attempt(retry_index);
        match attempt_once(state, prepared, &attempt, retry_index, execution_state, last_failure, last_error).await? {
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
    execution_state: &mut AttemptExecutionState,
    last_failure: &mut Option<transport::UpstreamFailure>,
    last_error: &mut Option<LlmProxyError>,
) -> Result<AttemptOnceOutcome, LlmProxyError> {
    if should_skip_codex_history_candidate(execution_state, candidate) {
        let error = codex_history_skip_error(execution_state);
        record_candidate_skipped_attempt(SkippedAttemptInput {
            state,
            request_id: &prepared.request_id,
            candidate,
            retry_index,
            skip_reason: SKIP_REASON_CODEX_CHAT_HISTORY_UNAVAILABLE,
            error: &error,
        })
        .await?;
        return Ok(AttemptOnceOutcome::ContinueCandidate);
    }
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
    let payload = match attempt_payload(AttemptContext::from_state(state), prepared.body.clone(), candidate, prepared.force_non_stream).await {
        Ok(payload) => payload,
        Err(error) => {
            if should_skip_codex_history_after_error(candidate, &error) {
                execution_state.codex_chat_history_unavailable_message = Some(error.to_string());
                record_skipped_attempt(SkippedAttemptInput {
                    state,
                    request_id: &prepared.request_id,
                    candidate,
                    retry_index,
                    skip_reason: SKIP_REASON_CODEX_CHAT_HISTORY_UNAVAILABLE,
                    error: &error,
                })
                .await?;
                *last_error = Some(error);
                return Ok(AttemptOnceOutcome::ContinueCandidate);
            }
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
    let response_headers_timeout = timeout::upstream_response_headers_timeout(candidate, prepared.is_stream);
    let response = match execute_upstream_request(&state.http, request, response_headers_timeout).await {
        Ok(response) => {
            attempt_cancel.mark_response_started();
            response
        }
        Err(error) => {
            attempt_cancel.disarm();
            let outcome = record_send_error(
                state,
                &prepared.request_id,
                candidate,
                retry_index,
                started,
                timeout_error_type(prepared.is_stream),
                &error,
                last_error,
            )
            .await?;
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

struct StreamAttemptTaskContext {
    state: LlmProxyState,
    request_id: String,
    cache_affinity_ttl_minutes: i64,
    candidate: ProxyCandidate,
    retry_index: i32,
    started: Instant,
    payload: AttemptPayload,
    attempt_cancel: AttemptCancelGuard,
}

async fn execute_stream_candidate_task(input: StreamAttemptTaskInput) -> Result<StreamAttemptTaskOutput, LlmProxyError> {
    let (context, request) = stream_task_parts(input);
    let response_headers_timeout = timeout::upstream_response_headers_timeout(&context.candidate, true);
    let response = match execute_upstream_request(&context.state.http, request, response_headers_timeout).await {
        Ok(response) => {
            context.attempt_cancel.mark_response_started();
            response
        }
        Err(error) => return stream_send_error_task_output(&context, error).await,
    };
    if !response.status().is_success() {
        return stream_status_failure_task_output(&context, response).await;
    }
    stream_success_task_output(context, response).await
}

fn stream_task_parts(input: StreamAttemptTaskInput) -> (StreamAttemptTaskContext, req::Request) {
    let StreamAttemptTaskInput {
        state,
        request_id,
        cache_affinity_ttl_minutes,
        candidate,
        retry_index,
        started,
        payload,
        request,
        attempt_cancel,
    } = input;
    (
        StreamAttemptTaskContext {
            state,
            request_id,
            cache_affinity_ttl_minutes,
            candidate,
            retry_index,
            started,
            payload,
            attempt_cancel,
        },
        request,
    )
}

async fn stream_send_error_task_output(context: &StreamAttemptTaskContext, error: req::ClientError) -> Result<StreamAttemptTaskOutput, LlmProxyError> {
    context.attempt_cancel.disarm();
    let mut last_error = None;
    let outcome = record_send_error(
        &context.state,
        &context.request_id,
        &context.candidate,
        context.retry_index,
        context.started,
        RESPONSE_HEADERS_TIMEOUT_ERROR_TYPE,
        &error,
        &mut last_error,
    )
    .await?;
    affinity::invalidate_matching(&context.state, &context.candidate).await?;
    Ok(StreamAttemptTaskOutput {
        outcome: stream_send_error_outcome(&error, outcome),
        last_failure: stream_send_error_last_failure(&error),
        last_error,
    })
}

async fn stream_status_failure_task_output(context: &StreamAttemptTaskContext, response: UpstreamResponse) -> Result<StreamAttemptTaskOutput, LlmProxyError> {
    let mut last_failure = None;
    let outcome = handle_upstream_failure(
        &context.state,
        &context.request_id,
        &context.candidate,
        context.retry_index,
        context.started,
        response,
        &mut last_failure,
    )
    .await;
    context.attempt_cancel.disarm();
    Ok(StreamAttemptTaskOutput {
        outcome: outcome?,
        last_failure,
        last_error: None,
    })
}

async fn stream_success_task_output(context: StreamAttemptTaskContext, response: UpstreamResponse) -> Result<StreamAttemptTaskOutput, LlmProxyError> {
    let stream_outcome = stream_transport::stream_response(
        stream_transport::StreamResponseArgs {
            state: context.state.clone(),
            request_id: context.request_id.clone(),
            response,
            candidate: context.candidate.clone(),
            source_format: context.payload.source_format,
            target_format: context.payload.target_format,
            provider_request_body: context.payload.body,
            started: context.started,
            retry_index: context.retry_index,
        },
        &context.attempt_cancel,
    )
    .await;
    match stream_outcome {
        Ok(stream_transport::StreamResponseOutcome::Response(response)) => {
            affinity::remember(&context.state, &context.candidate, context.cache_affinity_ttl_minutes).await?;
            Ok(StreamAttemptTaskOutput {
                outcome: AttemptOnceOutcome::Response(response),
                last_failure: None,
                last_error: None,
            })
        }
        Ok(stream_transport::StreamResponseOutcome::PreOutputFailure(failure)) => {
            affinity::invalidate_matching(&context.state, &context.candidate).await?;
            Ok(stream_pre_output_failure_task_output(failure))
        }
        Err(error) => Ok(StreamAttemptTaskOutput {
            outcome: AttemptOnceOutcome::ContinueCandidate,
            last_failure: None,
            last_error: Some(error),
        }),
    }
}

fn stream_pre_output_failure_task_output(failure: stream_transport::StreamPreOutputFailure) -> StreamAttemptTaskOutput {
    StreamAttemptTaskOutput {
        outcome: AttemptOnceOutcome::NextCandidate,
        last_failure: Some(transport::upstream_failure(failure.status)),
        last_error: Some(LlmProxyError::Upstream(format!("{}: {}", failure.error_type, failure.message))),
    }
}

fn stream_send_error_outcome(error: &req::ClientError, response: Option<Response>) -> AttemptOnceOutcome {
    if matches!(error, req::ClientError::Timeout) {
        return AttemptOnceOutcome::NextCandidate;
    }
    option_response_outcome(response)
}

fn stream_send_error_last_failure(error: &req::ClientError) -> Option<transport::UpstreamFailure> {
    matches!(error, req::ClientError::Timeout).then(transport::gateway_timeout_failure)
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

fn should_skip_codex_history_candidate(execution_state: &AttemptExecutionState, candidate: &ProxyCandidate) -> bool {
    execution_state.codex_chat_history_unavailable_message.is_some() && is_openai_cli_to_chat_candidate(candidate)
}

fn should_skip_codex_history_after_error(candidate: &ProxyCandidate, error: &LlmProxyError) -> bool {
    matches!(error, LlmProxyError::CodexChatHistoryUnavailable(_)) && is_openai_cli_to_chat_candidate(candidate)
}

fn is_openai_cli_to_chat_candidate(candidate: &ProxyCandidate) -> bool {
    candidate.trace.client_api_format == OPENAI_CLI_FORMAT && candidate.trace.provider_api_format == OPENAI_CHAT_FORMAT
}

fn codex_history_skip_error(execution_state: &AttemptExecutionState) -> LlmProxyError {
    let message = execution_state
        .codex_chat_history_unavailable_message
        .clone()
        .unwrap_or_else(|| "Codex chat history is unavailable for openai:cli to openai:chat conversion".into());
    LlmProxyError::CodexChatHistoryUnavailable(message)
}

async fn execute_upstream_request(
    http: &req::ReqwestClient,
    request: req::Request,
    response_headers_timeout: Option<Duration>,
) -> Result<UpstreamResponse, req::ClientError> {
    let execute = http.execute(request);
    match response_headers_timeout {
        Some(timeout) => tokio::time::timeout(timeout, execute).await.unwrap_or(Err(req::ClientError::Timeout)),
        None => execute.await,
    }
}

fn timeout_error_type(is_stream: bool) -> &'static str {
    if is_stream {
        return RESPONSE_HEADERS_TIMEOUT_ERROR_TYPE;
    }
    UPSTREAM_TIMEOUT_ERROR_TYPE
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
    debug_assert!(!input.prepared.is_stream, "stream responses must use the watchdog task path");
    let response = transport::FullResponseArgs {
        state: input.state,
        request_id: input.prepared.request_id.clone(),
        response: input.response,
        candidate: input.candidate.clone(),
        service_tier: input.prepared.service_tier.clone(),
        source_format: input.payload.source_format,
        target_format: input.payload.target_format,
        started: input.started,
        response_headers_time_ms: transport::elapsed_ms(input.started),
        retry_index: input.retry_index,
        request_timeout: input.request_timeout,
    };
    Ok(AttemptOnceOutcome::FullResponse(Box::new(response), Box::new(input.attempt_cancel)))
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod tests;
