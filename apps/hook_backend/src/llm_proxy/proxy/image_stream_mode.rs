use serde_json::Value;

use super::LlmProxyError;
use crate::llm_proxy::{OPENAI_IMAGE_EDIT_FORMAT, OPENAI_IMAGE_FORMAT, candidate::ProxyCandidate, formats};

pub(super) const IMAGE_STREAM_MODE_KEY: &str = "upstream_image_stream_mode";
pub(super) const IMAGE_STREAM_MODE_NATIVE: &str = "native_stream";
pub(super) const IMAGE_STREAM_MODE_SYNC_WRAPPED: &str = "sync_wrapped_stream";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ImageStreamMode {
    NativeStream,
    SyncWrappedStream,
}

impl ImageStreamMode {
    pub(super) fn upstream_is_stream(self) -> bool {
        matches!(self, Self::NativeStream)
    }
}

pub(super) fn candidate_image_stream_mode(candidate: &ProxyCandidate) -> Result<ImageStreamMode, LlmProxyError> {
    image_stream_mode_for_config(candidate.format_acceptance_config.as_ref())
}

pub(super) fn image_stream_mode_for_config(config: Option<&Value>) -> Result<ImageStreamMode, LlmProxyError> {
    let Some(object) = config.and_then(Value::as_object) else {
        return Ok(ImageStreamMode::SyncWrappedStream);
    };
    let Some(raw) = object.get(IMAGE_STREAM_MODE_KEY) else {
        return Ok(ImageStreamMode::SyncWrappedStream);
    };
    let Some(value) = raw.as_str() else {
        return Err(LlmProxyError::InvalidRequest(format!("{IMAGE_STREAM_MODE_KEY} must be a string")));
    };
    match value.trim() {
        IMAGE_STREAM_MODE_NATIVE => Ok(ImageStreamMode::NativeStream),
        IMAGE_STREAM_MODE_SYNC_WRAPPED => Ok(ImageStreamMode::SyncWrappedStream),
        other => Err(LlmProxyError::InvalidRequest(format!("unsupported {IMAGE_STREAM_MODE_KEY}: {other}"))),
    }
}

pub(super) fn is_openai_image_api_format(format: &str) -> bool {
    matches!(format, OPENAI_IMAGE_FORMAT | OPENAI_IMAGE_EDIT_FORMAT) || is_openai_image_metadata(format)
}

fn is_openai_image_metadata(format: &str) -> bool {
    formats::endpoint_metadata(format, false)
        .map(|metadata| matches!(metadata.kind, formats::EndpointKind::ImageGeneration | formats::EndpointKind::ImageEdit))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ImageStreamMode, image_stream_mode_for_config};

    #[test]
    fn image_stream_mode_defaults_to_sync_wrapped() {
        assert_eq!(image_stream_mode_for_config(None).unwrap(), ImageStreamMode::SyncWrappedStream);
        assert_eq!(image_stream_mode_for_config(Some(&json!({}))).unwrap(), ImageStreamMode::SyncWrappedStream);
    }

    #[test]
    fn image_stream_mode_parses_valid_values() {
        assert_eq!(
            image_stream_mode_for_config(Some(&json!({"upstream_image_stream_mode": "native_stream"}))).unwrap(),
            ImageStreamMode::NativeStream
        );
        assert_eq!(
            image_stream_mode_for_config(Some(&json!({"upstream_image_stream_mode": "sync_wrapped_stream"}))).unwrap(),
            ImageStreamMode::SyncWrappedStream
        );
    }

    #[test]
    fn image_stream_mode_rejects_invalid_values() {
        let error = image_stream_mode_for_config(Some(&json!({"upstream_image_stream_mode": "auto"}))).unwrap_err();

        assert_eq!(error.to_string(), "unsupported upstream_image_stream_mode: auto");
    }
}
