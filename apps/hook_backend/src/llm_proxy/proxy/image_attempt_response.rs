use std::time::{Duration, Instant};

use axum::{body::Body, http::StatusCode, response::Response};
use base64::Engine as _;
use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use super::{
    LlmProxyError, LlmProxyState, affinity,
    attempt_log::AttemptCancelGuard,
    failure_classification::{FailureDecision, classify_status},
    image_prepared::{OPENAI_IMAGE_API_FORMAT, PreparedImageRequest},
    image_record::{
        ImageRecordContext, ImageRecordContextInput, json_content_type, record_image_stream_success, record_image_sync_success, sse_content_type,
        usage_from_stream_summary,
    },
    image_response::normalize_image_response_bytes,
    stream_transport::{StreamResponseArgs, StreamResponseOutcome, stream_response},
    timeout, transport,
    transport_read::{ResponseBytesInput, response_bytes},
};
use crate::llm_proxy::candidate::ProxyCandidate;

pub(super) struct HandleResponseInput<'a> {
    pub(super) state: &'a LlmProxyState,
    pub(super) prepared: &'a PreparedImageRequest,
    pub(super) candidate: &'a ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) started: Instant,
    pub(super) upstream_is_stream: bool,
    pub(super) response: req::Response,
    pub(super) attempt_cancel: &'a AttemptCancelGuard,
    pub(super) last_failure: &'a mut Option<transport::UpstreamFailure>,
}

pub(super) enum ImageHandleResponseOutcome {
    ContinueCandidate,
    NextCandidate,
    Response(Response),
}

pub(super) async fn handle_response(input: HandleResponseInput<'_>) -> Result<ImageHandleResponseOutcome, LlmProxyError> {
    if !input.response.status().is_success() {
        return handle_upstream_failure(input).await;
    }
    let state = input.state;
    let candidate = input.candidate;
    let cache_affinity_ttl_minutes = input.prepared.cache_affinity_ttl_minutes;
    let response = if input.upstream_is_stream {
        handle_upstream_stream_success(input).await?
    } else {
        handle_upstream_sync_success(input).await?
    };
    affinity::remember(state, candidate, cache_affinity_ttl_minutes).await?;
    Ok(ImageHandleResponseOutcome::Response(response))
}

async fn handle_upstream_failure(input: HandleResponseInput<'_>) -> Result<ImageHandleResponseOutcome, LlmProxyError> {
    let decision = classify_status(input.response.status());
    let failure = transport::record_upstream_failure(
        input.state,
        &input.prepared.request_id,
        input.response,
        input.candidate,
        input.started,
        input.retry_index,
        decision.records_provider_cooldown(),
    )
    .await?;
    affinity::invalidate_retryable(input.state, input.candidate, decision).await?;
    classify_failure_outcome(decision, failure, input.last_failure)
}

async fn handle_upstream_stream_success(input: HandleResponseInput<'_>) -> Result<Response, LlmProxyError> {
    if input.prepared.is_stream {
        let outcome = stream_response(
            StreamResponseArgs {
                state: input.state.clone(),
                request_id: input.prepared.request_id.clone(),
                response: input.response,
                candidate: input.candidate.clone(),
                source_format: ApiFormat::OpenAiImage,
                target_format: ApiFormat::OpenAiImage,
                provider_request_body: input.prepared.body.provider_body(input.candidate, true)?,
                started: input.started,
                retry_index: input.retry_index,
            },
            input.attempt_cancel,
        )
        .await?;
        return stream_outcome_response(outcome);
    }
    upstream_stream_to_sync_response(input).await
}

async fn handle_upstream_sync_success(input: HandleResponseInput<'_>) -> Result<Response, LlmProxyError> {
    let timeout = timeout::non_stream_total_timeout(input.candidate, false);
    let record = image_record_context(&input);
    let report_context = input.prepared.body.report_context(input.candidate, &input.prepared.request_id);
    let bytes = read_success_bytes(
        input.state,
        input.prepared,
        input.candidate,
        input.retry_index,
        input.started,
        input.response,
        timeout,
    )
    .await?;
    let normalized = normalize_image_response_bytes(&input.state.http, &bytes).await?;
    if input.prepared.is_stream {
        return sync_json_to_stream_response(record, report_context, &bytes, &normalized).await;
    }
    record_image_sync_success(&record, &bytes, &normalized).await?;
    transport::response_builder(StatusCode::OK, Some(json_content_type()))
        .body(Body::from(normalized))
        .map_err(transport::response_error)
}

async fn upstream_stream_to_sync_response(input: HandleResponseInput<'_>) -> Result<Response, LlmProxyError> {
    let record = image_record_context(&input);
    let bytes = req::response_bytes(input.response).await?;
    let body = stream_bytes_to_sync_json(input.prepared, input.candidate, &bytes)?;
    let client = serde_json::to_vec(&body).map_err(json_error)?;
    record_image_sync_success(&record, &bytes, &client).await?;
    transport::response_builder(StatusCode::OK, Some(json_content_type()))
        .body(Body::from(client))
        .map_err(transport::response_error)
}

async fn sync_json_to_stream_response(
    record: ImageRecordContext,
    report_context: Value,
    provider_bytes: &[u8],
    client_bytes: &[u8],
) -> Result<Response, LlmProxyError> {
    let provider_body: Value = serde_json::from_slice(client_bytes).map_err(|error| LlmProxyError::Upstream(error.to_string()))?;
    let outcome =
        ::formats::api::maybe_bridge_standard_sync_json_to_stream(&provider_body, OPENAI_IMAGE_API_FORMAT, OPENAI_IMAGE_API_FORMAT, Some(&report_context))
            .map_err(format_error)?
            .ok_or_else(|| LlmProxyError::Upstream("upstream image response could not be wrapped as stream".into()))?;
    record_image_stream_success(
        &record,
        provider_bytes,
        &outcome.sse_body,
        usage_from_stream_summary(outcome.terminal_summary.as_ref()),
    )
    .await?;
    transport::response_builder(StatusCode::OK, Some(sse_content_type()))
        .body(Body::from(outcome.sse_body))
        .map_err(transport::response_error)
}

async fn read_success_bytes(
    state: &LlmProxyState,
    prepared: &PreparedImageRequest,
    candidate: &ProxyCandidate,
    retry_index: i32,
    started: Instant,
    response: req::Response,
    request_timeout: Option<Duration>,
) -> Result<Vec<u8>, LlmProxyError> {
    let read_timeout = request_timeout.map(|timeout| timeout::remaining_timeout(started, timeout));
    response_bytes(ResponseBytesInput {
        state,
        request_id: &prepared.request_id,
        candidate,
        retry_index,
        started,
        response_headers_time_ms: Some(transport::elapsed_ms(started)),
        first_token_time_ms: None,
        first_byte_time_ms: None,
        read_timeout,
        response,
    })
    .await
}

fn stream_bytes_to_sync_json(prepared: &PreparedImageRequest, candidate: &ProxyCandidate, bytes: &[u8]) -> Result<Value, LlmProxyError> {
    let body_base64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    let context = prepared.body.report_context(candidate, &prepared.request_id);
    let product = ::formats::api::maybe_build_openai_image_sync_finalize_product(
        ::formats::api::OPENAI_IMAGE_SYNC_FINALIZE_REPORT_KIND,
        200,
        Some(&context),
        None,
        Some(&body_base64),
    )
    .map_err(format_error)?
    .ok_or_else(|| LlmProxyError::Upstream("upstream image stream could not be finalized as JSON".into()))?;
    Ok(product.client_body_json)
}

fn stream_outcome_response(outcome: StreamResponseOutcome) -> Result<Response, LlmProxyError> {
    match outcome {
        StreamResponseOutcome::Response(response) => Ok(response),
        StreamResponseOutcome::PreOutputFailure(failure) => Err(LlmProxyError::Upstream(failure.message)),
    }
}

fn image_record_context(input: &HandleResponseInput<'_>) -> ImageRecordContext {
    ImageRecordContext::new(ImageRecordContextInput {
        state: input.state.clone(),
        request_id: input.prepared.request_id.clone(),
        candidate: input.candidate.clone(),
        retry_index: input.retry_index,
        started: input.started,
    })
}

fn classify_failure_outcome(
    decision: FailureDecision,
    failure: transport::UpstreamFailure,
    last_failure: &mut Option<transport::UpstreamFailure>,
) -> Result<ImageHandleResponseOutcome, LlmProxyError> {
    match decision {
        FailureDecision::ReturnResponse => transport::failure_response(failure).map(ImageHandleResponseOutcome::Response),
        FailureDecision::NextCandidate => {
            *last_failure = Some(failure);
            Ok(ImageHandleResponseOutcome::NextCandidate)
        }
        FailureDecision::RetryOrNextCandidate => {
            let cooldown_triggered = failure.cooldown_triggered();
            *last_failure = Some(failure);
            Ok(if cooldown_triggered {
                ImageHandleResponseOutcome::NextCandidate
            } else {
                ImageHandleResponseOutcome::ContinueCandidate
            })
        }
    }
}

fn format_error(error: impl std::fmt::Display) -> LlmProxyError {
    LlmProxyError::InvalidRequest(error.to_string())
}

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}
