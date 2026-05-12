use serde_json::Value;

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::{AttemptRecordInput, TokenUsage, record_attempt},
    candidate::ProxyCandidate,
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
    pub(super) first_byte_time_ms: Option<i64>,
    pub(super) error_type: Option<&'static str>,
    pub(super) error_message: Option<String>,
    pub(super) response_body: Option<Value>,
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
            response_body: input.response_body,
            finished: input.finished,
        },
    )
    .await
}

pub(super) fn response_read_error_type(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        return "upstream_timeout";
    }
    "upstream_response_read_error"
}
