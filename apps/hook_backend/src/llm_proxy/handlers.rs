use axum::{
    Json,
    extract::{Extension, Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::{Value, json};

use super::{
    CLAUDE_CHAT_FORMAT, CurrentApiToken, GEMINI_BATCH_EMBEDDING_FORMAT, GEMINI_CHAT_FORMAT, GEMINI_EMBEDDING_FORMAT, LlmProxyError, LlmProxyState,
    OPENAI_AUDIO_SPEECH_FORMAT, OPENAI_AUDIO_TRANSCRIPTION_FORMAT, OPENAI_AUDIO_TRANSLATION_FORMAT, OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT,
    OPENAI_COMPACT_FORMAT, OPENAI_COMPLETION_FORMAT, OPENAI_EMBEDDING_FORMAT, OPENAI_MODERATION_FORMAT, RERANK_FORMAT,
    model_access::{visible_model_for_token, visible_models_for_token},
    proxy::image::{proxy_image_edit, proxy_image_generation},
    proxy::{ProxyJsonRequest, proxy_json},
};

const OPENAI_MODEL_CREATED_AT: i64 = 1_626_777_600;

pub async fn chat_completions(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_CHAT_FORMAT, false)).await
}

pub async fn completions(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_COMPLETION_FORMAT, false)).await
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

pub async fn image_generations(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_image_generation(state, token, headers, body).await
}

pub async fn image_edits(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Response, LlmProxyError> {
    proxy_image_edit(state, token, headers, multipart).await
}

pub async fn image_variations() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": {
                "message": "API not implemented",
                "type": "new_api_error",
                "param": "",
                "code": "api_not_implemented"
            }
        })),
    )
        .into_response()
}

pub async fn embeddings(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_EMBEDDING_FORMAT, false)).await
}

pub async fn audio_transcriptions(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_AUDIO_TRANSCRIPTION_FORMAT, false)).await
}

pub async fn audio_translations(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_AUDIO_TRANSLATION_FORMAT, false)).await
}

pub async fn audio_speech(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_AUDIO_SPEECH_FORMAT, false)).await
}

pub async fn moderations(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, OPENAI_MODERATION_FORMAT, false)).await
}

pub async fn rerank(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, RERANK_FORMAT, false)).await
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

pub async fn gemini_embed_content(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    Path(model): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    let body = gemini_model_body(body, &model)?;
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, GEMINI_EMBEDDING_FORMAT, false)).await
}

pub async fn gemini_batch_embed_contents(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    Path(model): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, LlmProxyError> {
    let body = gemini_model_body(body, &model)?;
    proxy_json(ProxyJsonRequest::new(state, token, headers, body, GEMINI_BATCH_EMBEDDING_FORMAT, false)).await
}

pub async fn list_models(State(state): State<LlmProxyState>, Extension(token): Extension<CurrentApiToken>) -> Result<Json<OpenAiModelList>, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    let models = visible_models_for_token(&snapshot, &token.0)?.into_iter().map(openai_model).collect();
    Ok(Json(OpenAiModelList { object: "list", data: models }))
}

pub async fn retrieve_model(
    State(state): State<LlmProxyState>,
    Extension(token): Extension<CurrentApiToken>,
    Path(model): Path<String>,
) -> Result<Json<OpenAiModel>, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    let model = visible_model_for_token(&snapshot, &token.0, &model)?;
    Ok(Json(openai_model(model)))
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

fn gemini_model_body(mut body: Value, model: &str) -> Result<Value, LlmProxyError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    object.insert("model".into(), Value::String(model.to_owned()));
    Ok(body)
}

#[derive(Debug, PartialEq, Serialize)]
pub struct OpenAiModelList {
    object: &'static str,
    data: Vec<OpenAiModel>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct OpenAiModel {
    id: String,
    object: &'static str,
    created: i64,
    owned_by: &'static str,
}

fn openai_model(model: &crate::llm_proxy::cache::snapshot::CachedGlobalModel) -> OpenAiModel {
    OpenAiModel {
        id: model.name.clone(),
        object: "model",
        created: OPENAI_MODEL_CREATED_AT,
        owned_by: "hook",
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::model::TieredPricingConfig;

    use super::*;
    use crate::llm_proxy::cache::snapshot::CachedGlobalModel;

    #[test]
    fn openai_model_uses_global_model_name_as_id() {
        let model = CachedGlobalModel {
            id: "global-model-a".into(),
            name: "gpt-5".into(),
            is_active: true,
            supported_capabilities: None,
            default_price_per_request: Some(Decimal::ZERO),
            default_tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        };

        let response = openai_model(&model);

        assert_eq!(
            response,
            OpenAiModel {
                id: "gpt-5".into(),
                object: "model",
                created: OPENAI_MODEL_CREATED_AT,
                owned_by: "hook",
            }
        );
    }
}
