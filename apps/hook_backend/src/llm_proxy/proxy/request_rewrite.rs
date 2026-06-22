use proxy::format_conversion::ApiFormat;
use serde_json::{Map, Value};

use crate::llm_proxy::{LlmProxyError, candidate::ProxyCandidate, formats};

use super::image_stream_mode::{ImageStreamMode, candidate_image_stream_mode, is_openai_image_api_format};

const IMAGE_PARTIAL_IMAGES_FIELD: &str = "partial_images";
const STREAM_FIELD: &str = "stream";

pub(super) fn rewrite_upstream_body(body: &mut Value, candidate: &ProxyCandidate, force_non_stream: bool, target: ApiFormat) -> Result<(), LlmProxyError> {
    rewrite_upstream_body_with_stream(body, candidate, force_non_stream, target, None)
}

pub(super) fn rewrite_upstream_body_with_explicit_stream(
    body: &mut Value,
    candidate: &ProxyCandidate,
    upstream_is_stream: bool,
    target: ApiFormat,
) -> Result<(), LlmProxyError> {
    rewrite_upstream_body_with_stream(body, candidate, false, target, Some(upstream_is_stream))
}

fn rewrite_upstream_body_with_stream(
    body: &mut Value,
    candidate: &ProxyCandidate,
    force_non_stream: bool,
    target: ApiFormat,
    upstream_is_stream: Option<bool>,
) -> Result<(), LlmProxyError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    let metadata = formats::endpoint_metadata(
        &candidate.trace.provider_api_format,
        object.get(STREAM_FIELD).and_then(Value::as_bool).unwrap_or(false),
    )?;
    if metadata.model_in_body {
        object.insert("model".into(), Value::String(candidate.provider_model_name.clone()));
    } else {
        object.remove("model");
    }
    rewrite_stream_field(object, candidate, metadata, force_non_stream, upstream_is_stream)?;
    ensure_stream_usage(object, metadata, target, force_non_stream)?;
    Ok(())
}

fn rewrite_stream_field(
    object: &mut Map<String, Value>,
    candidate: &ProxyCandidate,
    metadata: formats::EndpointMetadata,
    force_non_stream: bool,
    upstream_is_stream: Option<bool>,
) -> Result<(), LlmProxyError> {
    if !metadata.stream_in_body || force_non_stream || metadata.upstream_stream_policy == formats::UpstreamStreamPolicy::ForceNonStream {
        object.remove(STREAM_FIELD);
        remove_image_stream_only_fields(object, candidate);
        return Ok(());
    }
    if !is_openai_image_api_format(&candidate.trace.provider_api_format) {
        return Ok(());
    }
    match image_stream_field_action(candidate, upstream_is_stream)? {
        ImageStreamFieldAction::Set(value) => {
            object.insert(STREAM_FIELD.into(), Value::Bool(value));
        }
        ImageStreamFieldAction::Remove => {
            object.remove(STREAM_FIELD);
            object.remove(IMAGE_PARTIAL_IMAGES_FIELD);
        }
        ImageStreamFieldAction::Keep => {}
    }
    Ok(())
}

fn remove_image_stream_only_fields(object: &mut Map<String, Value>, candidate: &ProxyCandidate) {
    if is_openai_image_api_format(&candidate.trace.provider_api_format) {
        object.remove(IMAGE_PARTIAL_IMAGES_FIELD);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageStreamFieldAction {
    Set(bool),
    Remove,
    Keep,
}

fn image_stream_field_action(candidate: &ProxyCandidate, upstream_is_stream: Option<bool>) -> Result<ImageStreamFieldAction, LlmProxyError> {
    if let Some(upstream_is_stream) = upstream_is_stream {
        return Ok(if upstream_is_stream {
            ImageStreamFieldAction::Set(true)
        } else {
            ImageStreamFieldAction::Remove
        });
    }
    Ok(match candidate_image_stream_mode(candidate)? {
        ImageStreamMode::NativeStream => ImageStreamFieldAction::Keep,
        ImageStreamMode::SyncWrappedStream => ImageStreamFieldAction::Remove,
    })
}

fn ensure_stream_usage(
    object: &mut Map<String, Value>,
    metadata: formats::EndpointMetadata,
    target: ApiFormat,
    force_non_stream: bool,
) -> Result<(), LlmProxyError> {
    if target != metadata.data_format
        || !metadata.include_usage_for_stream
        || force_non_stream
        || object.get(STREAM_FIELD).and_then(Value::as_bool) != Some(true)
    {
        return Ok(());
    }
    let stream_options = object.entry("stream_options").or_insert_with(|| Value::Object(Map::new()));
    let options = stream_options
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request field stream_options must be a JSON object".into()))?;
    options.insert("include_usage".into(), Value::Bool(true));
    Ok(())
}

pub(super) fn apply_reasoning_effort(body: &mut Value, candidate: &ProxyCandidate, target: ApiFormat) -> Result<(), LlmProxyError> {
    let Some(reasoning_effort) = candidate.reasoning_effort.as_deref() else {
        return Ok(());
    };
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    match target {
        ApiFormat::OpenAiChat => {
            object.insert("reasoning_effort".into(), Value::String(reasoning_effort.to_owned()));
            Ok(())
        }
        ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact => {
            reasoning_object(object)?.insert("effort".into(), Value::String(reasoning_effort.to_owned()));
            Ok(())
        }
        _ => Err(LlmProxyError::InvalidRequest(format!(
            "reasoning_effort override is not supported for provider format {}",
            candidate.trace.provider_api_format
        ))),
    }
}

fn reasoning_object(object: &mut Map<String, Value>) -> Result<&mut Map<String, Value>, LlmProxyError> {
    let value = object.entry("reasoning").or_insert_with(|| Value::Object(Map::new()));
    value
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request field reasoning must be a JSON object".into()))
}

#[cfg(test)]
#[path = "request_rewrite_tests.rs"]
mod tests;
