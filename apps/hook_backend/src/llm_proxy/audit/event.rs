use axum::http::HeaderMap;
use rust_decimal::Decimal;
use serde_json::Value;
use types::model::{PatchField, TieredPricingConfig};

use crate::llm_proxy::{
    candidate::{CandidateSelection, CandidateTrace, ProxyCandidate},
    proxy::capture::RequestCapture,
};

pub(super) enum AuditEvent<'a> {
    ScheduledCandidates {
        selection: &'a CandidateSelection,
        capture: &'a RequestCapture,
    },
    SkippedCandidates {
        request_id: &'a str,
        skip_reason: &'a str,
    },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TokenUsage {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub input_text_tokens: Option<i64>,
    pub input_audio_tokens: Option<i64>,
    pub input_image_tokens: Option<i64>,
    pub output_text_tokens: Option<i64>,
    pub output_audio_tokens: Option<i64>,
    pub output_image_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub cache_creation_5m_input_tokens: Option<i64>,
    pub cache_creation_1h_input_tokens: Option<i64>,
    pub usage_source: Option<&'static str>,
    pub usage_semantic: Option<&'static str>,
}

pub struct AttemptRecordInput<'a> {
    pub candidate: &'a ProxyCandidate,
    pub retry_index: i32,
    pub status: &'a str,
    pub skip_reason: Option<&'a str>,
    pub status_code: Option<i32>,
    pub usage: Option<TokenUsage>,
    pub service_tier: Option<String>,
    pub latency_ms: Option<i64>,
    pub response_headers_time_ms: Option<i64>,
    pub first_sse_event_time_ms: Option<i64>,
    pub first_token_time_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<&'a str>,
    pub error_message: Option<&'a str>,
    pub error_code: Option<&'a str>,
    pub error_param: Option<&'a str>,
    pub provider_request_headers: PatchField<HeaderMap>,
    pub provider_request_body: PatchField<Value>,
    pub provider_response_headers: PatchField<HeaderMap>,
    pub provider_response_body: PatchField<Value>,
    pub client_response_headers: PatchField<HeaderMap>,
    pub client_response_body: PatchField<Value>,
    pub termination_origin: PatchField<String>,
    pub termination_reason: PatchField<String>,
    pub stream_end_reason: PatchField<String>,
    pub finished: bool,
}

impl<'a> AttemptRecordInput<'a> {
    pub fn new(candidate: &'a ProxyCandidate, retry_index: i32, status: &'a str, finished: bool) -> Self {
        Self {
            candidate,
            retry_index,
            status,
            skip_reason: None,
            status_code: None,
            usage: None,
            service_tier: None,
            latency_ms: None,
            response_headers_time_ms: None,
            first_sse_event_time_ms: None,
            first_token_time_ms: None,
            first_byte_time_ms: None,
            error_type: None,
            error_message: None,
            error_code: None,
            error_param: None,
            provider_request_headers: PatchField::Missing,
            provider_request_body: PatchField::Missing,
            provider_response_headers: PatchField::Missing,
            provider_response_body: PatchField::Missing,
            client_response_headers: PatchField::Missing,
            client_response_body: PatchField::Missing,
            termination_origin: PatchField::Missing,
            termination_reason: PatchField::Missing,
            stream_end_reason: PatchField::Missing,
            finished,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AuditCandidate {
    pub(crate) trace: CandidateTrace,
    pub(crate) price_per_request: Option<Decimal>,
    pub(crate) tiered_pricing: TieredPricingConfig,
    pub(crate) billing_multiplier: Decimal,
}

#[derive(Clone, Debug)]
pub(crate) struct AttemptAuditInput {
    pub(crate) candidate: AuditCandidate,
    pub(crate) retry_index: i32,
    pub(crate) status: String,
    pub(crate) skip_reason: Option<String>,
    pub(crate) status_code: Option<i32>,
    pub(crate) usage: Option<TokenUsage>,
    pub(crate) service_tier: Option<String>,
    pub(crate) latency_ms: Option<i64>,
    pub(crate) response_headers_time_ms: Option<i64>,
    pub(crate) first_sse_event_time_ms: Option<i64>,
    pub(crate) first_token_time_ms: Option<i64>,
    pub(crate) first_byte_time_ms: Option<i64>,
    pub(crate) error_type: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_param: Option<String>,
    pub(crate) provider_request_headers: PatchField<HeaderMap>,
    pub(crate) provider_request_body: PatchField<Value>,
    pub(crate) provider_response_headers: PatchField<HeaderMap>,
    pub(crate) provider_response_body: PatchField<Value>,
    pub(crate) client_response_headers: PatchField<HeaderMap>,
    pub(crate) client_response_body: PatchField<Value>,
    pub(crate) termination_origin: PatchField<String>,
    pub(crate) termination_reason: PatchField<String>,
    pub(crate) stream_end_reason: PatchField<String>,
    pub(crate) finished: bool,
}

impl From<AttemptRecordInput<'_>> for AttemptAuditInput {
    fn from(input: AttemptRecordInput<'_>) -> Self {
        Self {
            candidate: AuditCandidate::from(input.candidate),
            retry_index: input.retry_index,
            status: input.status.to_owned(),
            skip_reason: input.skip_reason.map(str::to_owned),
            status_code: input.status_code,
            usage: input.usage,
            service_tier: input.service_tier,
            latency_ms: input.latency_ms,
            response_headers_time_ms: input.response_headers_time_ms,
            first_sse_event_time_ms: input.first_sse_event_time_ms,
            first_token_time_ms: input.first_token_time_ms,
            first_byte_time_ms: input.first_byte_time_ms,
            error_type: input.error_type.map(str::to_owned),
            error_message: input.error_message.map(str::to_owned),
            error_code: input.error_code.map(str::to_owned),
            error_param: input.error_param.map(str::to_owned),
            provider_request_headers: input.provider_request_headers,
            provider_request_body: input.provider_request_body,
            provider_response_headers: input.provider_response_headers,
            provider_response_body: input.provider_response_body,
            client_response_headers: input.client_response_headers,
            client_response_body: input.client_response_body,
            termination_origin: input.termination_origin,
            termination_reason: input.termination_reason,
            stream_end_reason: input.stream_end_reason,
            finished: input.finished,
        }
    }
}

impl From<&ProxyCandidate> for AuditCandidate {
    fn from(candidate: &ProxyCandidate) -> Self {
        Self {
            trace: candidate.trace.clone(),
            price_per_request: candidate.price_per_request,
            tiered_pricing: candidate.tiered_pricing.clone(),
            billing_multiplier: candidate.billing_multiplier,
        }
    }
}
