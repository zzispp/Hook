use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::response::Response;
use types::model::PatchField;

use super::{LlmProxyError, LlmProxyState};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, record_attempt, record_candidate_attempt},
    candidate::ProxyCandidate,
};

static CANDIDATE_CANCEL_REASONS: std::sync::OnceLock<Mutex<HashMap<(String, i32), AttemptCancelReason>>> = std::sync::OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AttemptCancelReason {
    ClientDisconnect,
    HedgedBackupSuperseded,
}

pub(super) struct AttemptCancelGuard {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    retry_index: i32,
    started: Instant,
    shared: Arc<Mutex<AttemptCancelShared>>,
}

impl AttemptCancelGuard {
    pub(super) fn handle(&self) -> AttemptCancelHandle {
        AttemptCancelHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    pub(super) fn mark_response_started(&self) {
        update_cancel_shared(&self.shared, |shared| {
            shared.phase = AttemptCancelPhase::AwaitingTerminal;
        });
    }

    pub(super) fn disarm(&self) {
        self.handle().disarm();
    }
}

impl Drop for AttemptCancelGuard {
    fn drop(&mut self) {
        let Some((phase, reason)) = take_cancel_state(&self.shared) else {
            return;
        };
        let reason = take_candidate_cancel_reason(&self.request_id, self.candidate.trace.candidate_index).unwrap_or(reason);
        let input = CancelledAttemptInput {
            state: self.state.clone(),
            request_id: self.request_id.clone(),
            candidate: self.candidate.clone(),
            retry_index: self.retry_index,
            latency_ms: elapsed_ms(self.started),
            phase,
            reason,
        };
        tokio::spawn(async move {
            if let Err(error) = record_cancelled_attempt(input).await {
                hook_tracing::warn_with_fields!("failed to record cancelled provider attempt", error = error);
            }
        });
    }
}

#[derive(Clone)]
pub(super) struct AttemptCancelHandle {
    shared: Arc<Mutex<AttemptCancelShared>>,
}

impl AttemptCancelHandle {
    pub(super) fn disarm(&self) {
        update_cancel_shared(&self.shared, |shared| {
            shared.armed = false;
        });
    }

    pub(super) fn reason(&self) -> AttemptCancelReason {
        self.shared.lock().map(|shared| shared.reason).unwrap_or(AttemptCancelReason::ClientDisconnect)
    }

    #[cfg(test)]
    pub(super) fn noop_for_test() -> Self {
        Self {
            shared: Arc::new(Mutex::new(AttemptCancelShared {
                phase: AttemptCancelPhase::WaitingUpstreamResponseStart,
                armed: false,
                reason: AttemptCancelReason::ClientDisconnect,
            })),
        }
    }
}

struct CancelledAttemptInput {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    retry_index: i32,
    latency_ms: i64,
    phase: AttemptCancelPhase,
    reason: AttemptCancelReason,
}

#[derive(Clone, Copy)]
enum AttemptCancelPhase {
    WaitingUpstreamResponseStart,
    AwaitingTerminal,
}

struct AttemptCancelShared {
    phase: AttemptCancelPhase,
    armed: bool,
    reason: AttemptCancelReason,
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

pub(super) struct SkippedAttemptInput<'a> {
    pub(super) state: &'a LlmProxyState,
    pub(super) request_id: &'a str,
    pub(super) candidate: &'a ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) skip_reason: &'static str,
    pub(super) error: &'a LlmProxyError,
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

pub(super) async fn record_skipped_attempt(input: SkippedAttemptInput<'_>) -> Result<(), LlmProxyError> {
    let error_message = input.error.to_string();
    record_attempt(input.state, input.request_id, skipped_attempt_record(&input, &error_message)).await
}

pub(super) async fn record_candidate_skipped_attempt(input: SkippedAttemptInput<'_>) -> Result<(), LlmProxyError> {
    let error_message = input.error.to_string();
    record_candidate_attempt(input.state, input.request_id, skipped_attempt_record(&input, &error_message)).await
}

fn skipped_attempt_record<'a>(input: &SkippedAttemptInput<'a>, error_message: &'a str) -> AttemptRecordInput<'a> {
    AttemptRecordInput {
        skip_reason: Some(input.skip_reason),
        error_type: Some(input.skip_reason),
        error_message: Some(error_message),
        ..AttemptRecordInput::new(input.candidate, input.retry_index, "skipped", true)
    }
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

pub(super) async fn record_probe_slot_timeout(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error: LlmProxyError,
    last_error: &mut Option<LlmProxyError>,
) -> Result<Option<Response>, LlmProxyError> {
    record_failed_attempt(state, request_id, candidate, retry_index, "provider_key_probe_slot_timeout", &error).await?;
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
        shared: Arc::new(Mutex::new(AttemptCancelShared {
            phase: AttemptCancelPhase::WaitingUpstreamResponseStart,
            armed: true,
            reason: AttemptCancelReason::ClientDisconnect,
        })),
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

pub(super) async fn record_stream_candidate_watchdog_timeout(
    state: &LlmProxyState,
    request_id: &str,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
) -> Result<(), LlmProxyError> {
    record_attempt(
        state,
        request_id,
        stream_candidate_watchdog_timeout_record(candidate, retry_index, elapsed_ms(started)),
    )
    .await
}

fn send_error_type(error: &req::ClientError) -> &'static str {
    if matches!(error, req::ClientError::Timeout) {
        return "upstream_timeout";
    }
    "upstream_send_error"
}

async fn record_cancelled_attempt(input: CancelledAttemptInput) -> Result<(), LlmProxyError> {
    let record = match input.reason {
        AttemptCancelReason::ClientDisconnect => match input.phase {
            AttemptCancelPhase::WaitingUpstreamResponseStart => {
                cancelled_before_upstream_response_record(&input.candidate, input.retry_index, input.latency_ms)
            }
            AttemptCancelPhase::AwaitingTerminal => cancelled_after_upstream_response_record(&input.candidate, input.retry_index, input.latency_ms),
        },
        AttemptCancelReason::HedgedBackupSuperseded => match input.phase {
            AttemptCancelPhase::WaitingUpstreamResponseStart => hedged_before_upstream_response_record(&input.candidate, input.retry_index, input.latency_ms),
            AttemptCancelPhase::AwaitingTerminal => hedged_after_upstream_response_record(&input.candidate, input.retry_index, input.latency_ms),
        },
    };
    record_attempt(&input.state, &input.request_id, record).await
}

pub(super) fn mark_candidate_cancel_reason(request_id: &str, candidate_index: i32, reason: AttemptCancelReason) {
    let map = CANDIDATE_CANCEL_REASONS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut map) = map.lock() {
        map.insert((request_id.to_owned(), candidate_index), reason);
    }
}

pub(super) fn take_candidate_cancel_reason(request_id: &str, candidate_index: i32) -> Option<AttemptCancelReason> {
    let map = CANDIDATE_CANCEL_REASONS.get_or_init(|| Mutex::new(HashMap::new()));
    map.lock().ok()?.remove(&(request_id.to_owned(), candidate_index))
}

pub(super) fn clear_candidate_cancel_reason(request_id: &str, candidate_index: i32) {
    let Some(map) = CANDIDATE_CANCEL_REASONS.get() else {
        return;
    };
    if let Ok(mut map) = map.lock() {
        map.remove(&(request_id.to_owned(), candidate_index));
    }
}

fn cancelled_before_upstream_response_record(candidate: &ProxyCandidate, retry_index: i32, latency_ms: i64) -> AttemptRecordInput<'_> {
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

fn cancelled_after_upstream_response_record(candidate: &ProxyCandidate, retry_index: i32, latency_ms: i64) -> AttemptRecordInput<'_> {
    AttemptRecordInput {
        status_code: Some(499),
        latency_ms: Some(latency_ms),
        error_type: Some("client_disconnected"),
        error_message: Some("client disconnected before request terminal finalization"),
        termination_origin: PatchField::Value("client".into()),
        termination_reason: PatchField::Value("disconnected".into()),
        stream_end_reason: PatchField::Value("client_gone".into()),
        ..AttemptRecordInput::new(candidate, retry_index, "cancelled", true)
    }
}

fn hedged_before_upstream_response_record(candidate: &ProxyCandidate, retry_index: i32, latency_ms: i64) -> AttemptRecordInput<'_> {
    AttemptRecordInput {
        latency_ms: Some(latency_ms),
        error_type: Some("hedge_cancelled"),
        error_message: Some("attempt cancelled before upstream response started because backup stream won"),
        termination_origin: PatchField::Value("gateway".into()),
        termination_reason: PatchField::Value("hedge_loser".into()),
        stream_end_reason: PatchField::Value("hedge_cancelled".into()),
        ..AttemptRecordInput::new(candidate, retry_index, "cancelled", true)
    }
}

fn hedged_after_upstream_response_record(candidate: &ProxyCandidate, retry_index: i32, latency_ms: i64) -> AttemptRecordInput<'_> {
    AttemptRecordInput {
        latency_ms: Some(latency_ms),
        error_type: Some("hedge_cancelled"),
        error_message: Some("attempt cancelled after upstream response started because backup stream won"),
        termination_origin: PatchField::Value("gateway".into()),
        termination_reason: PatchField::Value("hedge_loser".into()),
        stream_end_reason: PatchField::Value("hedge_cancelled".into()),
        ..AttemptRecordInput::new(candidate, retry_index, "cancelled", true)
    }
}

fn stream_candidate_watchdog_timeout_record(candidate: &ProxyCandidate, retry_index: i32, latency_ms: i64) -> AttemptRecordInput<'_> {
    AttemptRecordInput {
        status_code: Some(504),
        latency_ms: Some(latency_ms),
        error_type: Some("local_stream_candidate_watchdog_timeout"),
        error_message: Some("stream candidate timed out before handoff completed"),
        ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
    }
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

fn update_cancel_shared(shared: &Arc<Mutex<AttemptCancelShared>>, update: impl FnOnce(&mut AttemptCancelShared)) {
    if let Ok(mut shared) = shared.lock()
        && shared.armed
    {
        update(&mut shared);
    }
}

fn take_cancel_state(shared: &Arc<Mutex<AttemptCancelShared>>) -> Option<(AttemptCancelPhase, AttemptCancelReason)> {
    let mut shared = shared.lock().ok()?;
    if !shared.armed {
        return None;
    }
    shared.armed = false;
    Some((shared.phase, shared.reason))
}

#[cfg(test)]
#[path = "attempt_log_tests.rs"]
mod tests;
