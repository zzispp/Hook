use std::{collections::HashMap, time::Duration};

use axum::http::{HeaderMap, HeaderValue};
use types::model::PatchField;

use crate::llm_proxy::{
    InvalidateAffinityInput, LlmProxyError, LlmProxyState, REALTIME_PATH,
    audit::{AttemptRecordInput, SKIP_REASON_REQUEST_TERMINATED, record_attempt, record_skipped_candidates},
    candidate::{CandidateSelection, ProxyCandidate},
    rate_limit,
};

const OPENAI_REALTIME_BETA_HEADER: &str = "realtime=v1";

type UpstreamWs = req::WebSocketStream;
type UpstreamRequest = req::WebSocketRequest;

pub(super) struct ConnectedUpstream {
    pub(super) candidate: ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) request_headers: HeaderMap,
    pub(super) response_headers: HeaderMap,
    pub(super) stream: UpstreamWs,
}

pub(super) async fn connect_first_upstream(
    state: &LlmProxyState,
    selection: &CandidateSelection,
    query: &HashMap<String, String>,
) -> Result<ConnectedUpstream, LlmProxyError> {
    let mut last_error = None;
    for candidate in &selection.candidates {
        for retry_index in attempt_range(candidate) {
            let attempt = candidate.for_attempt(retry_index);
            if let Err(error @ LlmProxyError::RateLimited(_)) = rate_limit::claim_provider_key_limit(state, &attempt.trace.key_id, attempt.key_rpm_limit).await
            {
                record_connect_error(state, selection, &attempt, retry_index, None, &error).await?;
                last_error = Some(error);
                continue;
            }
            let request = match realtime_request(&attempt, query, attempt.api_key.clone()) {
                Ok(request) => request,
                Err(error) => {
                    record_connect_error(state, selection, &attempt, retry_index, None, &error).await?;
                    last_error = Some(error);
                    continue;
                }
            };
            let request_headers = request.headers().clone();
            match connect_upstream(&attempt, request).await {
                Ok((stream, response_headers)) => {
                    return Ok(ConnectedUpstream {
                        candidate: attempt,
                        retry_index,
                        request_headers,
                        response_headers,
                        stream,
                    });
                }
                Err(error) => {
                    record_connect_error(state, selection, &attempt, retry_index, Some(request_headers), &error).await?;
                    invalidate_connect_affinity(state, &attempt).await?;
                    last_error = Some(error);
                }
            }
        }
    }
    record_skipped_candidates(state, &selection.request_id, SKIP_REASON_REQUEST_TERMINATED).await?;
    Err(last_error.unwrap_or_else(|| LlmProxyError::Upstream("all realtime provider candidates failed".into())))
}

fn attempt_range(candidate: &ProxyCandidate) -> std::ops::RangeInclusive<i32> {
    0..=if candidate.is_cached { candidate.max_retries } else { 0 }
}

async fn invalidate_connect_affinity(state: &LlmProxyState, candidate: &ProxyCandidate) -> Result<(), LlmProxyError> {
    let Some(input) = invalidate_affinity_input(candidate) else {
        return Ok(());
    };
    state.invalidate_affinity(input).await
}

fn invalidate_affinity_input(candidate: &ProxyCandidate) -> Option<InvalidateAffinityInput<'_>> {
    Some(InvalidateAffinityInput {
        token_id: candidate.trace.token_id.as_deref()?,
        model_id: &candidate.trace.global_model_id,
        api_format: &candidate.trace.client_api_format,
        provider_id: &candidate.trace.provider_id,
        endpoint_id: &candidate.trace.endpoint_id,
        key_id: &candidate.trace.key_id,
    })
}

fn realtime_request(candidate: &ProxyCandidate, query: &HashMap<String, String>, api_key: String) -> Result<UpstreamRequest, LlmProxyError> {
    let url = realtime_url(candidate, query)?;
    req::build_websocket_request(url, realtime_headers(api_key)?).map_err(LlmProxyError::from)
}

fn realtime_url(candidate: &ProxyCandidate, query: &HashMap<String, String>) -> Result<req::Url, LlmProxyError> {
    let mut url = req::Url::parse(&realtime_base_url(candidate)).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid realtime url: {error}")))?;
    req::set_ws_scheme(&mut url).map_err(LlmProxyError::from)?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.clear();
        pairs.extend_pairs(query.iter().filter(|(key, _)| key.as_str() != "model"));
        pairs.append_pair("model", &candidate.provider_model_name);
    }
    Ok(url)
}

fn realtime_headers(api_key: String) -> Result<HeaderMap, LlmProxyError> {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(header_error)?);
    headers.insert("OpenAI-Beta", HeaderValue::from_static(OPENAI_REALTIME_BETA_HEADER));
    Ok(headers)
}

fn realtime_base_url(candidate: &ProxyCandidate) -> String {
    let base = candidate.base_url.trim_end_matches('/');
    let path = candidate
        .custom_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(REALTIME_PATH)
        .trim_start_matches('/');
    format!("{base}/{path}")
}

async fn connect_upstream(candidate: &ProxyCandidate, request: UpstreamRequest) -> Result<(UpstreamWs, HeaderMap), LlmProxyError> {
    req::connect_websocket(request, candidate.stream_first_byte_timeout_seconds.and_then(timeout_duration))
        .await
        .map_err(LlmProxyError::from)
}

fn timeout_duration(seconds: f64) -> Option<Duration> {
    (seconds.is_finite() && seconds > 0.0).then(|| Duration::from_secs_f64(seconds))
}

async fn record_connect_error(
    state: &LlmProxyState,
    selection: &CandidateSelection,
    candidate: &ProxyCandidate,
    retry_index: i32,
    request_headers: Option<HeaderMap>,
    error: &LlmProxyError,
) -> Result<(), LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        &selection.request_id,
        AttemptRecordInput {
            error_type: Some(connect_error_type(error)),
            error_message: Some(error_message.as_str()),
            provider_request_headers: request_headers.map(PatchField::Value).unwrap_or(PatchField::Missing),
            ..AttemptRecordInput::new(candidate, retry_index, "failed", true)
        },
    )
    .await
}

fn connect_error_type(error: &LlmProxyError) -> &'static str {
    if error.to_string().contains("timed out") {
        return "upstream_timeout";
    }
    "upstream_connect_error"
}

fn header_error(error: axum::http::header::InvalidHeaderValue) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}
