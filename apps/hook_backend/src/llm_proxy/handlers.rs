use axum::{
    Json,
    extract::{Extension, Path, State},
    http::HeaderMap,
    response::Response,
};
use serde_json::Value;

use super::{
    CLAUDE_CHAT_FORMAT, CurrentApiToken, GEMINI_CHAT_FORMAT, LlmProxyError, LlmProxyState, OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT, OPENAI_COMPACT_FORMAT,
    proxy::{ProxyJsonRequest, proxy_json},
};

pub async fn chat_completions(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_CHAT_FORMAT, false)).await
}

pub async fn responses(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_CLI_FORMAT, false)).await
}

pub async fn responses_compact(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_COMPACT_FORMAT, true)).await
}

pub async fn claude_messages(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, CLAUDE_CHAT_FORMAT, false)).await
}

pub async fn gemini_generate_content(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    Path(model_action): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    let (model, action) = gemini_model_action(&model_action)?;
    let body = gemini_body(body, model, action)?;
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, GEMINI_CHAT_FORMAT, false)).await
}

fn gemini_model_action(value: &str) -> Result<(&str, &str), LlmProxyError> {
    let Some((model, action)) = value.split_once(':') else {
        return Err(LlmProxyError::InvalidRequest("Gemini route must be /v1beta/models/{model}:{action}".into()));
    };
    if model.trim().is_empty() {
        return Err(LlmProxyError::InvalidRequest("Gemini model path segment cannot be blank".into()));
    }
    if !matches!(action, "generateContent" | "streamGenerateContent") {
        return Err(LlmProxyError::InvalidRequest(format!("unsupported Gemini action: {action}")));
    }
    Ok((model, action))
}

fn gemini_body(mut body: Value, model: &str, action: &str) -> Result<Value, LlmProxyError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    object.insert("model".into(), Value::String(model.to_owned()));
    if action == "streamGenerateContent" {
        object.insert("stream".into(), Value::Bool(true));
    }
    Ok(body)
}
