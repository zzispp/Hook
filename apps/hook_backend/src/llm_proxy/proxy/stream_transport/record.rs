use serde_json::Value;
use types::model::PatchField;

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::{AttemptRecordInput, TokenUsage, record_attempt},
    candidate::ProxyCandidate,
};

use super::{
    StreamAttemptContext,
    status::StreamStatus,
    terminal::{StreamCooldownFailure, StreamTerminalObservability, StreamTerminalSummary},
};

pub(super) struct StreamAttemptRecord {
    pub(super) state: LlmProxyState,
    pub(super) request_id: String,
    pub(super) candidate: ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) status: &'static str,
    pub(super) status_code: Option<i32>,
    pub(super) usage: Option<TokenUsage>,
    pub(super) latency_ms: Option<i64>,
    pub(super) response_headers_time_ms: Option<i64>,
    pub(super) first_sse_event_time_ms: Option<i64>,
    pub(super) first_output_time_ms: Option<i64>,
    pub(super) first_byte_time_ms: Option<i64>,
    pub(super) error_type: Option<&'static str>,
    pub(super) error_message: Option<String>,
    pub(super) provider_response_body: PatchField<Value>,
    pub(super) client_response_body: PatchField<Value>,
    pub(super) termination_origin: PatchField<String>,
    pub(super) termination_reason: PatchField<String>,
    pub(super) stream_end_reason: PatchField<String>,
    pub(super) finished: bool,
}

pub(super) async fn record_stream_attempt(input: StreamAttemptRecord) -> Result<(), LlmProxyError> {
    record_attempt(
        &input.state,
        &input.request_id,
        AttemptRecordInput {
            candidate: &input.candidate,
            retry_index: input.retry_index,
            status: input.status,
            status_code: input.status_code,
            usage: input.usage,
            latency_ms: input.latency_ms,
            response_headers_time_ms: input.response_headers_time_ms,
            first_sse_event_time_ms: input.first_sse_event_time_ms,
            first_output_time_ms: input.first_output_time_ms,
            first_byte_time_ms: input.first_byte_time_ms,
            error_type: input.error_type,
            error_message: input.error_message.as_deref(),
            provider_response_body: input.provider_response_body,
            client_response_body: input.client_response_body,
            termination_origin: input.termination_origin,
            termination_reason: input.termination_reason,
            stream_end_reason: input.stream_end_reason,
            ..AttemptRecordInput::new(&input.candidate, input.retry_index, input.status, input.finished)
        },
    )
    .await
}

pub(super) struct StreamTerminalRecordInput<'a> {
    pub(super) context: &'a StreamAttemptContext,
    pub(super) usage: Option<TokenUsage>,
    pub(super) summary: StreamTerminalSummary,
}

pub(super) fn terminal_stream_record(input: StreamTerminalRecordInput<'_>) -> StreamAttemptRecord {
    let summary = input.summary;
    let mut record = terminal_record(TerminalRecordInput {
        context: input.context,
        status: summary.record_status,
        status_code: Some(summary.status_code),
        usage: input.usage,
        observability: summary.observability.clone(),
        error_type: summary.error_type,
        error_message: summary.error_message,
    });
    record.termination_origin = summary.termination_origin;
    record.termination_reason = summary.termination_reason;
    record.stream_end_reason = summary.stream_end_reason;
    record
}

pub(super) struct StreamCancelledRecordInput<'a> {
    pub(super) context: &'a StreamAttemptContext,
    pub(super) usage: Option<TokenUsage>,
    pub(super) status: &'a StreamStatus,
    pub(super) observability: StreamTerminalObservability,
    pub(super) reason: StreamCancelledReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StreamCancelledReason {
    ClientDisconnected,
    HedgedBackupSuperseded,
}

pub(super) fn cancelled_record(input: StreamCancelledRecordInput<'_>) -> StreamAttemptRecord {
    let (status_code, error_type, error_message, termination_origin, termination_reason) = match input.reason {
        StreamCancelledReason::ClientDisconnected => (
            Some(499),
            "client_disconnected",
            "client disconnected before stream completed",
            "client",
            "disconnected",
        ),
        StreamCancelledReason::HedgedBackupSuperseded => (
            None,
            "hedge_cancelled",
            "stream attempt cancelled because backup stream won",
            "gateway",
            "hedge_loser",
        ),
    };
    StreamAttemptRecord {
        termination_origin: PatchField::Value(termination_origin.into()),
        termination_reason: PatchField::Value(termination_reason.into()),
        stream_end_reason: input.status.stream_end_reason_patch(),
        ..terminal_record(TerminalRecordInput {
            context: input.context,
            status: "cancelled",
            status_code,
            usage: input.usage,
            observability: input.observability,
            error_type: Some(error_type),
            error_message: Some(error_message.into()),
        })
    }
}

struct TerminalRecordInput<'a> {
    context: &'a StreamAttemptContext,
    status: &'static str,
    status_code: Option<i32>,
    usage: Option<TokenUsage>,
    observability: StreamTerminalObservability,
    error_type: Option<&'static str>,
    error_message: Option<String>,
}

fn terminal_record(input: TerminalRecordInput<'_>) -> StreamAttemptRecord {
    StreamAttemptRecord {
        state: input.context.state.clone(),
        request_id: input.context.request_id.clone(),
        candidate: input.context.candidate.clone(),
        retry_index: input.context.retry_index,
        status: input.status,
        status_code: input.status_code,
        usage: input.usage,
        latency_ms: Some(input.observability.latency_ms),
        response_headers_time_ms: input.observability.response_headers_time_ms,
        first_sse_event_time_ms: input.observability.first_sse_event_time_ms,
        first_output_time_ms: input.observability.first_output_time_ms,
        first_byte_time_ms: input.observability.first_byte_time_ms,
        error_type: input.error_type,
        error_message: input.error_message,
        provider_response_body: input.observability.bodies.provider_response_body,
        client_response_body: input.observability.bodies.client_response_body,
        termination_origin: PatchField::Null,
        termination_reason: PatchField::Null,
        stream_end_reason: PatchField::Null,
        finished: true,
    }
}

pub(super) fn response_read_error_type(error: &req::ClientError) -> &'static str {
    if matches!(error, req::ClientError::Timeout) {
        return "stream_idle_timeout";
    }
    "upstream_response_read_error"
}

pub(super) async fn record_stream_cooldown(
    context: &StreamAttemptContext,
    cooldown: Option<StreamCooldownFailure>,
    message: &str,
) -> Result<(), LlmProxyError> {
    let Some(cooldown) = cooldown else {
        return Ok(());
    };
    context
        .state
        .record_provider_status_failure(crate::llm_proxy::cache::ProviderCooldownFailureInput {
            request_id: &context.request_id,
            candidate: &context.candidate,
            retry_index: context.retry_index,
            status_code: cooldown.status_code,
            error_type: cooldown.error_type,
            error_message: message,
            error_code: None,
            error_param: None,
        })
        .await?;
    Ok(())
}
