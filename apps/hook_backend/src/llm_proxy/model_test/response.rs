use std::collections::BTreeMap;

use axum::{
    body::to_bytes,
    http::{HeaderMap, StatusCode},
};
use serde_json::Value;
use storage::provider::ProviderStore;
use types::provider::{ProviderModelTestEndpoint, ProviderModelTestResponse, RequestCandidateDetail};

use super::TestRequest;
use crate::llm_proxy::{LlmProxyError, LlmProxyState};

pub(super) async fn response_to_result(response: axum::response::Response, request: &TestRequest) -> Result<ProviderModelTestResponse, LlmProxyError> {
    let status = response.status();
    let headers = response_headers(response.headers());
    let body = response_body(response).await?;
    let detail = request_detail(&request.state, &request.selection.request_id).await?;
    let candidate = last_attempt(&detail.candidates);
    Ok(ProviderModelTestResponse {
        success: status.is_success(),
        model: request.model_name.clone(),
        endpoint: test_endpoint(request, candidate),
        status_code: status_code(candidate, status)?,
        latency_ms: candidate.and_then(|item| item.latency_ms).unwrap_or_default() as u128,
        request_url: request_url(request, candidate),
        request_body: candidate.and_then(|item| item.provider_request_body.clone()).unwrap_or(Value::Null),
        response_headers: headers,
        response_body: body,
        error: test_error(status, candidate, &detail.record.client_error_message),
    })
}

fn status_code(candidate: Option<&RequestCandidateDetail>, status: StatusCode) -> Result<Option<u16>, LlmProxyError> {
    let Some(value) = candidate.and_then(|item| item.status_code) else {
        return Ok(Some(status.as_u16()));
    };
    u16::try_from(value)
        .map(Some)
        .map_err(|_| LlmProxyError::Infrastructure(format!("invalid recorded upstream status code: {value}")))
}

async fn response_body(response: axum::response::Response) -> Result<Value, LlmProxyError> {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
    Ok(body_value(&bytes))
}

async fn request_detail(state: &LlmProxyState, request_id: &str) -> Result<types::provider::RequestRecordDetail, LlmProxyError> {
    ProviderStore::new(state.database.clone())
        .get_request_record(request_id)
        .await
        .map_err(Into::into)
}

fn last_attempt(candidates: &[RequestCandidateDetail]) -> Option<&RequestCandidateDetail> {
    candidates
        .iter()
        .rfind(|candidate| candidate.status != "scheduled" && candidate.status != "skipped")
}

fn test_error(status: StatusCode, candidate: Option<&RequestCandidateDetail>, client_error: &Option<String>) -> Option<String> {
    if status.is_success() {
        return None;
    }
    candidate
        .and_then(|item| item.error_message.clone())
        .or_else(|| client_error.clone())
        .or_else(|| Some(format!("provider model test failed with status {}", status.as_u16())))
}

fn response_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| Some((name.as_str().to_owned(), value.to_str().ok()?.to_owned())))
        .collect()
}

fn body_value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap_or_else(|_| String::from_utf8(bytes.to_vec()).map(Value::String).unwrap_or(Value::Null))
}

fn test_endpoint(request: &TestRequest, candidate: Option<&RequestCandidateDetail>) -> ProviderModelTestEndpoint {
    candidate
        .and_then(|item| item.endpoint_id.as_deref())
        .and_then(|endpoint_id| route_endpoint(request, endpoint_id))
        .unwrap_or_else(|| request.endpoint.clone())
}

fn request_url(request: &TestRequest, candidate: Option<&RequestCandidateDetail>) -> String {
    candidate
        .and_then(|item| item.endpoint_id.as_deref())
        .and_then(|endpoint_id| route_url(request, endpoint_id))
        .unwrap_or_else(|| request.request_url.clone())
}

fn route_endpoint(request: &TestRequest, endpoint_id: &str) -> Option<ProviderModelTestEndpoint> {
    route_option(request, endpoint_id).map(|option| ProviderModelTestEndpoint {
        id: option.endpoint.id.clone(),
        api_format: option.endpoint.provider_api_format.clone(),
        base_url: option.endpoint.base_url.clone(),
    })
}

fn route_url(request: &TestRequest, endpoint_id: &str) -> Option<String> {
    route_option(request, endpoint_id).map(|option| option.endpoint.upstream_url.clone())
}

fn route_option<'a>(request: &'a TestRequest, endpoint_id: &str) -> Option<&'a crate::llm_proxy::candidate::CandidateRouteOption> {
    request
        .selection
        .candidates
        .iter()
        .flat_map(|candidate| candidate.route.options.iter())
        .find(|option| option.endpoint.id == endpoint_id)
}
