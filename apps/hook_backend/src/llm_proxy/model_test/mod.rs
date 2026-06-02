mod candidate;
mod response;
mod selection;

use std::collections::BTreeMap;

use async_trait::async_trait;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use provider::application::{ProviderError, ProviderModelTester, ProviderResult};
use serde_json::Value;
use types::provider::{ProviderModelTestEndpoint, ProviderModelTestRequest, ProviderModelTestResponse};
use uuid::Uuid;

use super::{
    LlmProxyError, LlmProxyState,
    candidate::CandidateSelection,
    proxy::{ProxyFixedJsonRequest, proxy_fixed_json},
};

const TESTABLE_FORMATS: &[&str] = &[
    "openai:chat",
    "openai:cli",
    "openai:compact",
    "openai_image",
    "openai_image_edit",
    "claude:chat",
    "gemini:cli",
];
const TEST_GROUP_CODE: &str = "admin_model_test";

pub(crate) struct LlmProxyProviderModelTester {
    state: LlmProxyState,
}

impl LlmProxyProviderModelTester {
    pub(crate) fn new(state: LlmProxyState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ProviderModelTester for LlmProxyProviderModelTester {
    async fn test_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelTestRequest) -> ProviderResult<ProviderModelTestResponse> {
        run_test(self.state.clone(), provider_id, model_id, input).await.map_err(provider_error)
    }
}

struct TestRequest {
    state: LlmProxyState,
    headers: HeaderMap,
    body: Value,
    selection: CandidateSelection,
    model_name: String,
    endpoint: ProviderModelTestEndpoint,
    request_url: String,
    client_api_format: String,
    force_non_stream: bool,
}

impl TestRequest {
    async fn new(state: LlmProxyState, provider_id: &str, model_id: &str, input: ProviderModelTestRequest) -> Result<Self, LlmProxyError> {
        let headers = header_map(&input.request_headers)?;
        let body = input.request_body;
        let requested_stream = is_stream(&body);
        let snapshot = state.scheduling_snapshot().await?;
        let parts = selection::fixed_parts(&snapshot, provider_id, model_id, &input.endpoint_id, &input.key_id, requested_stream)?;
        let force_non_stream = parts.force_non_stream;
        let effective_stream = parts.effective_stream;
        let client_api_format = parts.client_api_format.clone();
        let selection = CandidateSelection {
            request_id: Uuid::now_v7().to_string(),
            cache_affinity_ttl_minutes: snapshot.cache_affinity_ttl_minutes,
            candidates: vec![candidate::proxy_candidate(&state, parts.clone(), effective_stream)?],
        };
        let primary = primary_endpoint(&selection)?;
        Ok(Self {
            state,
            headers,
            body,
            selection,
            model_name: parts.model.provider_model_name,
            endpoint: primary.endpoint,
            request_url: primary.request_url,
            client_api_format,
            force_non_stream,
        })
    }
}

async fn run_test(
    state: LlmProxyState,
    provider_id: &str,
    model_id: &str,
    input: ProviderModelTestRequest,
) -> Result<ProviderModelTestResponse, LlmProxyError> {
    let request = TestRequest::new(state, provider_id, model_id, input).await?;
    let response = proxy_fixed_json(ProxyFixedJsonRequest::new(
        request.state.clone(),
        request.headers.clone(),
        request.body.clone(),
        request.client_api_format.clone(),
        request.force_non_stream,
        request.selection.clone(),
    ))
    .await?;
    response::response_to_result(response, &request).await
}

fn header_map(headers: &BTreeMap<String, String>) -> Result<HeaderMap, LlmProxyError> {
    let mut output = HeaderMap::new();
    for (key, value) in headers {
        output.insert(header_name(key)?, header_value(key, value)?);
    }
    Ok(output)
}

fn header_name(value: &str) -> Result<HeaderName, LlmProxyError> {
    HeaderName::from_bytes(value.trim().as_bytes()).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid test header name {value:?}: {error}")))
}

fn header_value(key: &str, value: &str) -> Result<HeaderValue, LlmProxyError> {
    HeaderValue::from_str(value).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid test header value for {key:?}: {error}")))
}

fn is_stream(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(false)
}

struct PrimaryEndpoint {
    endpoint: ProviderModelTestEndpoint,
    request_url: String,
}

fn primary_endpoint(selection: &CandidateSelection) -> Result<PrimaryEndpoint, LlmProxyError> {
    let option = selection
        .candidates
        .first()
        .and_then(|candidate| candidate.route.options.first())
        .ok_or_else(|| LlmProxyError::Infrastructure("model test candidate route is empty".into()))?;
    Ok(PrimaryEndpoint {
        endpoint: ProviderModelTestEndpoint {
            id: option.endpoint.id.clone(),
            api_format: option.endpoint.provider_api_format.clone(),
            base_url: option.endpoint.base_url.clone(),
        },
        request_url: option.endpoint.upstream_url.clone(),
    })
}

fn provider_error(error: LlmProxyError) -> ProviderError {
    match error {
        LlmProxyError::InvalidRequest(message) | LlmProxyError::NotFound(message) => ProviderError::InvalidInput(message),
        other => ProviderError::Infrastructure(other.to_string()),
    }
}
