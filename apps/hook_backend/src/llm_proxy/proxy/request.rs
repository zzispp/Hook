use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::Value;
use types::api_token::ApiToken;

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::record_available_candidates,
    candidate::{CandidateRequest, ProxyCandidate, select_candidates},
    formats,
};

use super::capture::RequestCapture;

pub(super) struct PreparedProxyRequest {
    pub(super) request_id: String,
    pub(super) candidates: Vec<ProxyCandidate>,
    pub(super) body: Value,
    pub(super) is_stream: bool,
    pub(super) force_non_stream: bool,
}

pub(super) struct AttemptPayload {
    pub(super) body: Value,
    pub(super) source_format: ApiFormat,
    pub(super) target_format: ApiFormat,
}

pub(super) async fn prepare_proxy_request(
    state: &LlmProxyState,
    token: &ApiToken,
    body: Value,
    api_format: &'static str,
    force_non_stream: bool,
    capture: RequestCapture,
) -> Result<PreparedProxyRequest, LlmProxyError> {
    let model_name = required_model(&body)?;
    let is_stream = is_streaming(&body) && !force_non_stream;
    let selection = select_candidates(
        state,
        token,
        CandidateRequest {
            api_format,
            model_name,
            is_stream,
        },
    )
    .await?;
    record_available_candidates(state, &selection, &capture).await?;
    Ok(PreparedProxyRequest {
        request_id: selection.request_id,
        candidates: selection.candidates,
        body,
        is_stream,
        force_non_stream,
    })
}

pub(super) fn attempt_payload(body: Value, candidate: &ProxyCandidate, force_non_stream: bool) -> Result<AttemptPayload, LlmProxyError> {
    let (body, source_format, target_format) = upstream_body(body, candidate, force_non_stream)?;
    Ok(AttemptPayload {
        body,
        source_format,
        target_format,
    })
}

fn required_model(body: &Value) -> Result<&str, LlmProxyError> {
    body.get("model")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must include a non-empty model".into()))
}

fn is_streaming(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(false)
}

fn upstream_body(body: Value, candidate: &ProxyCandidate, force_non_stream: bool) -> Result<(Value, ApiFormat, ApiFormat), LlmProxyError> {
    let mut body = body;
    let source = formats::parse_api_format(candidate.trace.client_api_format.as_str())?;
    let target = formats::parse_api_format(candidate.trace.provider_api_format.as_str())?;
    if candidate.trace.needs_conversion {
        body = FormatConversionRegistry::default()
            .convert_request(&body, source, target)
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
    }
    rewrite_upstream_body(&mut body, candidate, force_non_stream, target)?;
    Ok((body, source, target))
}

fn rewrite_upstream_body(body: &mut Value, candidate: &ProxyCandidate, force_non_stream: bool, target: ApiFormat) -> Result<(), LlmProxyError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    object.insert("model".into(), Value::String(candidate.provider_model_name.clone()));
    if force_non_stream {
        object.remove("stream");
    }
    if target == ApiFormat::GeminiChat {
        object.remove("model");
        object.remove("stream");
    }
    Ok(())
}
