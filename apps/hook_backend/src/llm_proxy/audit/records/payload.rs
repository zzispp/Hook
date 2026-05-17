use axum::http::HeaderMap;
use serde_json::Value;
use types::model::PatchField;

use crate::llm_proxy::{
    LlmProxyError,
    proxy::capture::{recorded_headers, recorded_request_body},
    request_record_policy::RequestRecordPolicy,
};

pub(super) fn header_patch(headers: PatchField<HeaderMap>, policy: &RequestRecordPolicy) -> Result<PatchField<Value>, LlmProxyError> {
    match headers {
        PatchField::Value(headers) => Ok(option_patch(recorded_headers(&headers, policy))),
        PatchField::Null => Ok(PatchField::Null),
        PatchField::Missing => Ok(PatchField::Missing),
    }
}

pub(super) fn header_input(headers: PatchField<HeaderMap>, policy: &RequestRecordPolicy) -> Option<Value> {
    match headers {
        PatchField::Value(headers) => recorded_headers(&headers, policy),
        PatchField::Null | PatchField::Missing => None,
    }
}

pub(super) fn request_body_patch(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<PatchField<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => Ok(option_patch(recorded_request_body(&body, policy).map_err(infra_error)?)),
        PatchField::Null => Ok(PatchField::Null),
        PatchField::Missing => Ok(PatchField::Missing),
    }
}

pub(super) fn request_body_input(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<Option<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => recorded_request_body(&body, policy).map_err(infra_error),
        PatchField::Null | PatchField::Missing => Ok(None),
    }
}

pub(super) fn response_body_patch(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<PatchField<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => Ok(option_patch(policy.response_body(Some(body)).map_err(infra_error)?)),
        PatchField::Null => Ok(PatchField::Null),
        PatchField::Missing => Ok(PatchField::Missing),
    }
}

pub(super) fn response_body_input(body: PatchField<Value>, policy: &RequestRecordPolicy) -> Result<Option<Value>, LlmProxyError> {
    match body {
        PatchField::Value(body) => policy.response_body(Some(body)).map_err(infra_error),
        PatchField::Null | PatchField::Missing => Ok(None),
    }
}

fn option_patch<T>(value: Option<T>) -> PatchField<T> {
    match value {
        Some(value) => PatchField::Value(value),
        None => PatchField::Null,
    }
}

fn infra_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}
