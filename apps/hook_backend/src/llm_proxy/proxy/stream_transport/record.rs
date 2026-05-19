use serde_json::Value;
use types::model::PatchField;

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::{AttemptRecordInput, TokenUsage, record_attempt},
    candidate::ProxyCandidate,
    proxy::transport,
};

use super::{StreamAttemptContext, status::StreamStatus};

pub(super) struct StreamAttemptRecord {
    pub(super) state: LlmProxyState,
    pub(super) request_id: String,
    pub(super) candidate: ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) status: &'static str,
    pub(super) status_code: Option<i32>,
    pub(super) usage: Option<TokenUsage>,
    pub(super) latency_ms: Option<i64>,
    pub(super) first_byte_time_ms: Option<i64>,
    pub(super) error_type: Option<&'static str>,
    pub(super) error_message: Option<String>,
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
            first_byte_time_ms: input.first_byte_time_ms,
            error_type: input.error_type,
            error_message: input.error_message.as_deref(),
            client_response_body: input.client_response_body,
            termination_origin: input.termination_origin,
            termination_reason: input.termination_reason,
            stream_end_reason: input.stream_end_reason,
            ..AttemptRecordInput::new(&input.candidate, input.retry_index, input.status, input.finished)
        },
    )
    .await
}

pub(super) fn streaming_record(context: &StreamAttemptContext, first_byte_elapsed: i64) -> StreamAttemptRecord {
    StreamAttemptRecord {
        state: context.state.clone(),
        request_id: context.request_id.clone(),
        candidate: context.candidate.clone(),
        retry_index: context.retry_index,
        status: "streaming",
        status_code: Some(context.status.as_u16() as i32),
        usage: None,
        latency_ms: None,
        first_byte_time_ms: Some(first_byte_elapsed),
        error_type: None,
        error_message: None,
        client_response_body: PatchField::Missing,
        termination_origin: PatchField::Missing,
        termination_reason: PatchField::Missing,
        stream_end_reason: PatchField::Missing,
        finished: false,
    }
}

pub(super) fn success_record(
    context: &StreamAttemptContext,
    usage: Option<TokenUsage>,
    first_byte_time_ms: Option<i64>,
    status: &StreamStatus,
) -> StreamAttemptRecord {
    let mut record = terminal_record(TerminalRecordInput {
        context,
        status: "success",
        status_code: Some(context.status.as_u16() as i32),
        usage,
        first_byte_time_ms,
        error_type: None,
        error_message: None,
    });
    record.stream_end_reason = status.stream_end_reason_patch();
    record
}

pub(super) fn failure_record(
    context: &StreamAttemptContext,
    first_byte_time_ms: Option<i64>,
    error_type: &'static str,
    error_message: &str,
    status: &StreamStatus,
) -> StreamAttemptRecord {
    let mut record = terminal_record(TerminalRecordInput {
        context,
        status: "failed",
        status_code: Some(context.status.as_u16() as i32),
        usage: None,
        first_byte_time_ms,
        error_type: Some(error_type),
        error_message: Some(error_message.to_owned()),
    });
    record.stream_end_reason = status.stream_end_reason_patch();
    record
}

pub(super) fn cancelled_record(
    context: &StreamAttemptContext,
    usage: Option<TokenUsage>,
    first_byte_time_ms: Option<i64>,
    status: &StreamStatus,
) -> StreamAttemptRecord {
    StreamAttemptRecord {
        termination_origin: PatchField::Value("client".into()),
        termination_reason: PatchField::Value("disconnected".into()),
        stream_end_reason: status.stream_end_reason_patch(),
        ..terminal_record(TerminalRecordInput {
            context,
            status: "cancelled",
            status_code: Some(499),
            usage,
            first_byte_time_ms,
            error_type: Some("client_disconnected"),
            error_message: Some("client disconnected before stream completed".into()),
        })
    }
}

struct TerminalRecordInput<'a> {
    context: &'a StreamAttemptContext,
    status: &'static str,
    status_code: Option<i32>,
    usage: Option<TokenUsage>,
    first_byte_time_ms: Option<i64>,
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
        latency_ms: Some(transport::elapsed_ms(input.context.started)),
        first_byte_time_ms: input.first_byte_time_ms,
        error_type: input.error_type,
        error_message: input.error_message,
        client_response_body: PatchField::Missing,
        termination_origin: PatchField::Null,
        termination_reason: PatchField::Null,
        stream_end_reason: PatchField::Null,
        finished: true,
    }
}

pub(super) fn response_read_error_type(error: &req::ClientError) -> &'static str {
    if matches!(error, req::ClientError::Timeout) {
        return "upstream_timeout";
    }
    "upstream_response_read_error"
}
