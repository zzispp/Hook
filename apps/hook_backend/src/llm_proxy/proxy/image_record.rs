use std::time::Instant;

use axum::http::{HeaderMap, HeaderValue, header};
use proxy::format_conversion::ApiFormat;
use serde_json::Value;
use types::model::PatchField;

use super::{LlmProxyError, LlmProxyState, response_payload::body_value, transport, usage};
use crate::llm_proxy::{
    audit::{AttemptRecordInput, TokenUsage, record_attempt},
    candidate::ProxyCandidate,
};

const STREAM_END_REASON_DONE: &str = "done";

pub(super) struct ImageRecordContext {
    state: LlmProxyState,
    request_id: String,
    candidate: ProxyCandidate,
    retry_index: i32,
    started: Instant,
}

pub(super) struct ImageRecordContextInput {
    pub(super) state: LlmProxyState,
    pub(super) request_id: String,
    pub(super) candidate: ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) started: Instant,
}

impl ImageRecordContext {
    pub(super) fn new(input: ImageRecordContextInput) -> Self {
        Self {
            state: input.state,
            request_id: input.request_id,
            candidate: input.candidate,
            retry_index: input.retry_index,
            started: input.started,
        }
    }
}

pub(super) async fn record_image_sync_success(input: &ImageRecordContext, provider_bytes: &[u8], client_bytes: &[u8]) -> Result<(), LlmProxyError> {
    record_image_success(
        input,
        provider_bytes,
        client_bytes,
        usage::from_response_bytes(client_bytes, ApiFormat::OpenAiImage),
        false,
    )
    .await
}

pub(super) async fn record_image_stream_success(
    input: &ImageRecordContext,
    provider_bytes: &[u8],
    client_bytes: &[u8],
    usage: Option<TokenUsage>,
) -> Result<(), LlmProxyError> {
    record_image_success(input, provider_bytes, client_bytes, usage, true).await
}

pub(super) fn response_headers(is_stream: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, if is_stream { sse_content_type() } else { json_content_type() });
    headers
}

pub(super) fn json_content_type() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

pub(super) fn sse_content_type() -> HeaderValue {
    HeaderValue::from_static("text/event-stream")
}

async fn record_image_success(
    input: &ImageRecordContext,
    provider_bytes: &[u8],
    client_bytes: &[u8],
    usage: Option<TokenUsage>,
    is_stream: bool,
) -> Result<(), LlmProxyError> {
    record_attempt(
        &input.state,
        &input.request_id,
        AttemptRecordInput {
            status_code: Some(200),
            usage,
            latency_ms: Some(transport::elapsed_ms(input.started)),
            first_byte_time_ms: Some(transport::elapsed_ms(input.started)),
            provider_response_body: PatchField::Value(body_value(provider_bytes)),
            client_response_headers: PatchField::Value(response_headers(is_stream)),
            client_response_body: PatchField::Value(body_value(client_bytes)),
            stream_end_reason: stream_end_reason_patch(is_stream),
            ..AttemptRecordInput::new(&input.candidate, input.retry_index, "success", true)
        },
    )
    .await
}

fn stream_end_reason_patch(is_stream: bool) -> PatchField<String> {
    if is_stream {
        return PatchField::Value(STREAM_END_REASON_DONE.into());
    }
    PatchField::Missing
}

pub(super) fn usage_from_stream_summary(summary: Option<&::formats::contracts::ExecutionStreamTerminalSummary>) -> Option<TokenUsage> {
    let usage = summary?.standardized_usage.as_ref()?;
    Some(TokenUsage {
        prompt_tokens: positive_usage(usage.input_tokens),
        completion_tokens: positive_usage(usage.output_tokens),
        total_tokens: usage.dimensions.get("total_tokens").and_then(Value::as_i64),
        cache_creation_input_tokens: positive_usage(usage.cache_creation_tokens),
        cache_read_input_tokens: positive_usage(usage.cache_read_tokens),
        input_text_tokens: usage.dimensions.get("input_text_tokens").and_then(Value::as_i64),
        input_image_tokens: usage.dimensions.get("input_image_tokens").and_then(Value::as_i64),
        output_image_tokens: usage.dimensions.get("image_count").and_then(Value::as_i64),
        usage_source: Some("openai"),
        usage_semantic: Some("image"),
        ..TokenUsage::default()
    })
}

fn positive_usage(value: i64) -> Option<i64> {
    (value > 0).then_some(value)
}
