use axum::{http::HeaderMap, response::Response};
use serde_json::Value;
use types::{api_token::ApiToken, provider::RoutingRequestFeatures};

use super::{
    LlmProxyError, LlmProxyState,
    capture::RequestCapture,
    image_attempt::{execute_sync_client_response, spawn_stream_image_attempts},
    image_form::MultipartImageRequest,
    image_prepared::{PreparedImageRequest, PreparedImageRequestBody},
    image_stream_mode::candidate_image_stream_mode,
    image_stream_wrapper::{self, StreamImageRequest},
};
use crate::llm_proxy::{
    CurrentApiToken, IMAGE_GENERATION_CAPABILITY, OPENAI_IMAGE_EDIT_FORMAT, OPENAI_IMAGE_FORMAT,
    audit::record_scheduled_candidates,
    billing::enforce_preflight_access,
    candidate::{CandidateRequest, select_candidates},
    rate_limit,
};

pub(super) async fn execute_image_generation_request(
    state: LlmProxyState,
    token: CurrentApiToken,
    headers: HeaderMap,
    body: Value,
) -> Result<Response, LlmProxyError> {
    let capture = RequestCapture::new(&headers, &body);
    let request = PreparedImageRequestBody::Json { body };
    let prepared = prepare_image_request(&state, &token.0, OPENAI_IMAGE_FORMAT, request, capture).await?;
    execute_prepared_image_request(state, prepared).await
}

pub(super) async fn execute_image_edit_request(
    state: LlmProxyState,
    token: CurrentApiToken,
    headers: HeaderMap,
    request: MultipartImageRequest,
) -> Result<Response, LlmProxyError> {
    let capture = RequestCapture::new(&headers, request.record_body());
    let prepared = prepare_image_request(
        &state,
        &token.0,
        OPENAI_IMAGE_EDIT_FORMAT,
        PreparedImageRequestBody::Multipart(request),
        capture,
    )
    .await?;
    execute_prepared_image_request(state, prepared).await
}

impl PreparedImageRequest {
    fn into_stream_request(self) -> StreamImageRequest {
        StreamImageRequest {
            request_id: self.request_id,
            cache_affinity_ttl_minutes: self.cache_affinity_ttl_minutes,
            candidates: self.candidates,
            body: self.body,
        }
    }
}

impl From<StreamImageRequest> for PreparedImageRequest {
    fn from(request: StreamImageRequest) -> Self {
        Self {
            request_id: request.request_id,
            cache_affinity_ttl_minutes: request.cache_affinity_ttl_minutes,
            candidates: request.candidates,
            body: request.body,
            is_stream: true,
        }
    }
}

async fn prepare_image_request(
    state: &LlmProxyState,
    token: &ApiToken,
    api_format: &'static str,
    body: PreparedImageRequestBody,
    capture: RequestCapture,
) -> Result<PreparedImageRequest, LlmProxyError> {
    enforce_preflight_access(state, token).await?;
    rate_limit::enforce_request_limits(state, token).await?;
    let is_stream = body.is_stream();
    let selection = select_candidates(
        state,
        token,
        CandidateRequest {
            api_format,
            routing_api_format: api_format,
            model_name: body.model(),
            is_stream,
            has_openai_responses_custom_tool_items: false,
            features: RoutingRequestFeatures::unknown(api_format, is_stream, Some(IMAGE_GENERATION_CAPABILITY)),
        },
    )
    .await?;
    record_scheduled_candidates(state, &selection, &capture).await?;
    Ok(PreparedImageRequest {
        request_id: selection.request_id,
        cache_affinity_ttl_minutes: selection.cache_affinity_ttl_minutes,
        candidates: selection.candidates,
        body,
        is_stream,
    })
}

async fn execute_prepared_image_request(state: LlmProxyState, prepared: PreparedImageRequest) -> Result<Response, LlmProxyError> {
    if prepared.is_stream && stream_response_needs_sync_wrapper(&prepared)? {
        return image_stream_wrapper::stream_client_response(state, prepared.into_stream_request(), spawn_stream_image_attempts).await;
    }
    execute_sync_client_response(state, prepared).await
}

fn stream_response_needs_sync_wrapper(prepared: &PreparedImageRequest) -> Result<bool, LlmProxyError> {
    prepared.candidates.iter().try_fold(false, |needs_wrapper, candidate| {
        Ok(needs_wrapper || !candidate_image_stream_mode(candidate)?.upstream_is_stream())
    })
}
