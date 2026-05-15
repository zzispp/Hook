use axum::http::HeaderMap;
use serde_json::Value;
use types::model::PatchField;

use crate::llm_proxy::{
    candidate::{CandidateSelection, ProxyCandidate},
    proxy::capture::RequestCapture,
};

pub(super) enum AuditEvent<'a> {
    ScheduledCandidates {
        selection: &'a CandidateSelection,
        capture: &'a RequestCapture,
    },
    Attempt {
        request_id: &'a str,
        input: Box<AttemptRecordInput<'a>>,
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
