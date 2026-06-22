use axum::{extract::Multipart, http::HeaderMap, response::Response};
use serde_json::{Map, Number, Value};

use super::{
    image_executor::{execute_image_edit_request, execute_image_generation_request},
    image_form::MultipartImageRequest,
};
use crate::llm_proxy::{CurrentApiToken, LlmProxyError, LlmProxyState};

pub(super) const DEFAULT_IMAGE_COUNT: u64 = 1;
pub(super) const DEFAULT_IMAGE_SIZE: &str = "1024x1024";
pub(super) const DEFAULT_DALLE_3_QUALITY: &str = "standard";
pub(super) const DEFAULT_GPT_IMAGE_EDIT_QUALITY: &str = "standard";
pub(super) const DEFAULT_GPT_IMAGE_JSON_QUALITY: &str = "auto";
const DALLE_2_SIZES: &[&str] = &["256x256", "512x512", "1024x1024"];
const DALLE_3_SIZES: &[&str] = &["1024x1024", "1024x1792", "1792x1024"];

pub(crate) async fn proxy_image_generation(state: LlmProxyState, token: CurrentApiToken, headers: HeaderMap, body: Value) -> Result<Response, LlmProxyError> {
    let body = validate_image_json(body)?;
    execute_image_generation_request(state, token, headers, body).await
}

pub(crate) async fn proxy_image_edit(
    state: LlmProxyState,
    token: CurrentApiToken,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Response, LlmProxyError> {
    let request = MultipartImageRequest::from_multipart(multipart).await?;
    execute_image_edit_request(state, token, headers, request).await
}

pub(super) fn validate_image_json(mut body: Value) -> Result<Value, LlmProxyError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    normalize_json_fields(object, DEFAULT_GPT_IMAGE_JSON_QUALITY)?;
    Ok(body)
}

fn normalize_json_fields(object: &mut Map<String, Value>, gpt_image_quality: &str) -> Result<(), LlmProxyError> {
    let model = required_object_string(object, "model")?.to_owned();
    required_object_string(object, "prompt")?;
    set_count_field(object, normalize_count_value(object.get("n"))?)?;
    set_optional_text(object, "size", normalize_size(&model, optional_object_string(object, "size"))?);
    set_optional_text(
        object,
        "quality",
        normalize_quality(&model, optional_object_string(object, "quality"), gpt_image_quality),
    );
    Ok(())
}

pub(super) fn normalize_size(model: &str, size: Option<&str>) -> Result<Option<String>, LlmProxyError> {
    let size = match size.map(str::trim).filter(|value| !value.is_empty()) {
        Some(size) => size,
        None if matches!(model, "dall-e-2" | "dall-e" | "dall-e-3") => return Ok(Some(DEFAULT_IMAGE_SIZE.to_owned())),
        None => return Ok(None),
    };
    if size.contains('×') {
        return Err(LlmProxyError::InvalidRequest(
            "size an unexpected error occurred in the parameter, please use 'x' instead of the multiplication sign '×'".into(),
        ));
    }
    validate_size(model, size)?;
    Ok(Some(size.to_owned()))
}

fn validate_size(model: &str, size: &str) -> Result<(), LlmProxyError> {
    if matches!(model, "dall-e-2" | "dall-e") && !DALLE_2_SIZES.contains(&size) {
        return Err(LlmProxyError::InvalidRequest(
            "size must be one of 256x256, 512x512, or 1024x1024 for dall-e-2 or dall-e".into(),
        ));
    }
    if model == "dall-e-3" && !DALLE_3_SIZES.contains(&size) {
        return Err(LlmProxyError::InvalidRequest(
            "size must be one of 1024x1024, 1024x1792 or 1792x1024 for dall-e-3".into(),
        ));
    }
    Ok(())
}

pub(super) fn normalize_quality(model: &str, quality: Option<&str>, gpt_image_quality: &str) -> Option<String> {
    quality
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| (model == "dall-e-3").then(|| DEFAULT_DALLE_3_QUALITY.to_owned()))
        .or_else(|| (model == "gpt-image-1").then(|| gpt_image_quality.to_owned()))
}

fn normalize_count_value(value: Option<&Value>) -> Result<u64, LlmProxyError> {
    match value {
        None | Some(Value::Null) => Ok(DEFAULT_IMAGE_COUNT),
        Some(Value::Number(number)) => Ok(number.as_u64().filter(|value| *value > 0).unwrap_or(DEFAULT_IMAGE_COUNT)),
        Some(Value::String(text)) => normalize_count_text(Some(text)),
        Some(_) => Err(LlmProxyError::InvalidRequest("request field n must be a positive integer".into())),
    }
}

pub(super) fn normalize_count_text(value: Option<&str>) -> Result<u64, LlmProxyError> {
    let value = value.map(str::trim).filter(|value| !value.is_empty()).unwrap_or("1");
    Ok(value.parse::<u64>().ok().filter(|parsed| *parsed > 0).unwrap_or(DEFAULT_IMAGE_COUNT))
}

fn required_object_string<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str, LlmProxyError> {
    optional_object_string(object, key)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmProxyError::InvalidRequest(format!("{key} is required")))
}

fn optional_object_string<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

fn set_count_field(object: &mut Map<String, Value>, count: u64) -> Result<(), LlmProxyError> {
    let count = i64::try_from(count).map_err(|_| LlmProxyError::InvalidRequest("request field n is too large".into()))?;
    object.insert("n".into(), Value::Number(Number::from(count)));
    Ok(())
}

fn set_optional_text(object: &mut Map<String, Value>, key: &str, value: Option<String>) {
    match value {
        Some(value) => {
            object.insert(key.to_owned(), Value::String(value));
        }
        None => {
            object.remove(key);
        }
    }
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod tests;
