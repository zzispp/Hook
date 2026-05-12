use std::{collections::HashMap, time::Duration};

use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::client::IntoClientRequest};

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState, REALTIME_PATH,
    audit::{AttemptRecordInput, record_attempt},
    candidate::{CandidateSelection, ProxyCandidate},
};

const OPENAI_REALTIME_BETA_HEADER: &str = "realtime=v1";

type UpstreamWs = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub(super) struct ConnectedUpstream {
    pub(super) candidate: ProxyCandidate,
    pub(super) retry_index: i32,
    pub(super) stream: UpstreamWs,
}

pub(super) async fn connect_first_upstream(
    state: &LlmProxyState,
    selection: &CandidateSelection,
    query: &HashMap<String, String>,
) -> Result<ConnectedUpstream, LlmProxyError> {
    let mut last_error = None;
    for candidate in &selection.candidates {
        for retry_index in 0..=candidate.max_retries {
            match connect_upstream(candidate, realtime_url(candidate, query)?, candidate.api_key.clone()).await {
                Ok(stream) => {
                    return Ok(ConnectedUpstream {
                        candidate: candidate.clone(),
                        retry_index,
                        stream,
                    });
                }
                Err(error) => {
                    record_connect_error(state, selection, candidate, retry_index, &error).await?;
                    last_error = Some(error);
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| LlmProxyError::Upstream("all realtime provider candidates failed".into())))
}

fn realtime_url(candidate: &ProxyCandidate, query: &HashMap<String, String>) -> Result<reqwest::Url, LlmProxyError> {
    let mut url =
        reqwest::Url::parse(&realtime_base_url(candidate)).map_err(|error| LlmProxyError::InvalidRequest(format!("invalid realtime url: {error}")))?;
    set_ws_scheme(&mut url)?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.clear();
        pairs.extend_pairs(query.iter().filter(|(key, _)| key.as_str() != "model"));
        pairs.append_pair("model", &candidate.provider_model_name);
    }
    Ok(url)
}

fn set_ws_scheme(url: &mut reqwest::Url) -> Result<(), LlmProxyError> {
    let scheme = match url.scheme() {
        "http" => "ws",
        "https" => "wss",
        "ws" | "wss" => return Ok(()),
        other => return Err(LlmProxyError::InvalidRequest(format!("unsupported upstream scheme for websocket: {other}"))),
    };
    url.set_scheme(scheme)
        .map_err(|_| LlmProxyError::InvalidRequest("failed to build websocket upstream url".into()))
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

async fn connect_upstream(candidate: &ProxyCandidate, url: reqwest::Url, api_key: String) -> Result<UpstreamWs, LlmProxyError> {
    let mut request = url.as_str().into_client_request().map_err(|error| LlmProxyError::Upstream(error.to_string()))?;
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {api_key}").parse().map_err(header_error)?);
    request
        .headers_mut()
        .insert("OpenAI-Beta", OPENAI_REALTIME_BETA_HEADER.parse().map_err(header_error)?);
    let connect = connect_async(request);
    let result = match candidate.stream_first_byte_timeout_seconds.and_then(timeout_duration) {
        Some(timeout) => tokio::time::timeout(timeout, connect)
            .await
            .map_err(|_| LlmProxyError::Upstream("upstream websocket connect timed out".into()))?,
        None => connect.await,
    };
    let (stream, _) = result.map_err(|error| LlmProxyError::Upstream(error.to_string()))?;
    Ok(stream)
}

fn timeout_duration(seconds: f64) -> Option<Duration> {
    (seconds.is_finite() && seconds > 0.0).then(|| Duration::from_secs_f64(seconds))
}

async fn record_connect_error(
    state: &LlmProxyState,
    selection: &CandidateSelection,
    candidate: &ProxyCandidate,
    retry_index: i32,
    error: &LlmProxyError,
) -> Result<(), LlmProxyError> {
    let error_message = error.to_string();
    record_attempt(
        state,
        &selection.request_id,
        AttemptRecordInput {
            candidate,
            retry_index,
            status: "failed",
            status_code: None,
            usage: None,
            latency_ms: None,
            first_byte_time_ms: None,
            error_type: Some(connect_error_type(error)),
            error_message: Some(error_message.as_str()),
            response_body: None,
            finished: true,
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
