use proxy::format_conversion::ApiFormat;
use serde_json::{Value, json};

use super::{LlmProxyError, image_form::MultipartImageRequest, request::rewrite_upstream_body_with_explicit_stream};
use crate::llm_proxy::candidate::ProxyCandidate;

pub(super) const OPENAI_IMAGE_API_FORMAT: &str = "openai:image";

pub(super) struct PreparedImageRequest {
    pub(super) request_id: String,
    pub(super) cache_affinity_ttl_minutes: i64,
    pub(super) candidates: Vec<ProxyCandidate>,
    pub(super) body: PreparedImageRequestBody,
    pub(super) is_stream: bool,
}

#[derive(Clone)]
pub(super) enum PreparedImageRequestBody {
    Json { body: Value },
    Multipart(MultipartImageRequest),
}

pub(super) enum UpstreamImageBody {
    Json(Value),
    Multipart(req::multipart::Form),
}

impl PreparedImageRequestBody {
    pub(super) fn model(&self) -> &str {
        match self {
            Self::Json { body } => body.get("model").and_then(Value::as_str).unwrap_or_default(),
            Self::Multipart(request) => request.model(),
        }
    }

    pub(super) fn original_body(&self) -> &Value {
        match self {
            Self::Json { body } => body,
            Self::Multipart(request) => request.record_body(),
        }
    }

    pub(super) fn provider_body(&self, candidate: &ProxyCandidate, upstream_is_stream: bool) -> Result<Value, LlmProxyError> {
        match self {
            Self::Json { body } => json_provider_body(body, candidate, upstream_is_stream),
            Self::Multipart(request) => Ok(multipart_provider_body(request, candidate, upstream_is_stream)),
        }
    }

    pub(super) fn upstream_body(&self, candidate: &ProxyCandidate, upstream_is_stream: bool) -> Result<UpstreamImageBody, LlmProxyError> {
        match self {
            Self::Json { body } => Ok(UpstreamImageBody::Json(json_provider_body(body, candidate, upstream_is_stream)?)),
            Self::Multipart(request) => Ok(UpstreamImageBody::Multipart(
                request.build_form(&candidate.provider_model_name, upstream_is_stream)?,
            )),
        }
    }

    pub(super) fn report_context(&self, candidate: &ProxyCandidate, request_id: &str) -> Value {
        let operation = match self {
            Self::Json { .. } => "generate",
            Self::Multipart(_) => "edit",
        };
        json!({
            "request_id": request_id,
            "client_api_format": OPENAI_IMAGE_API_FORMAT,
            "provider_api_format": OPENAI_IMAGE_API_FORMAT,
            "mapped_model": candidate.provider_model_name,
            "model": candidate.requested_model_name,
            "image_request": image_request_context(self.original_body(), operation),
        })
    }

    pub(super) fn is_stream(&self) -> bool {
        match self {
            Self::Json { body } => body.get("stream").and_then(Value::as_bool).unwrap_or(false),
            Self::Multipart(request) => request.is_stream(),
        }
    }
}

fn json_provider_body(body: &Value, candidate: &ProxyCandidate, upstream_is_stream: bool) -> Result<Value, LlmProxyError> {
    let mut body = body.clone();
    rewrite_upstream_body_with_explicit_stream(&mut body, candidate, upstream_is_stream, ApiFormat::OpenAiImage)?;
    Ok(body)
}

fn multipart_provider_body(request: &MultipartImageRequest, candidate: &ProxyCandidate, upstream_is_stream: bool) -> Value {
    let mut body = request.provider_body(&candidate.provider_model_name);
    if let Some(object) = body.as_object_mut() {
        if upstream_is_stream {
            object.insert("stream".into(), Value::String("true".into()));
        } else {
            object.remove("stream");
        }
    }
    body
}

fn image_request_context(body: &Value, operation: &str) -> Value {
    json!({
        "operation": operation,
        "output_format": body.get("output_format").and_then(Value::as_str),
        "response_format": body.get("response_format").and_then(Value::as_str),
        "size": body.get("size").and_then(Value::as_str),
        "quality": body.get("quality").and_then(Value::as_str),
        "partial_images": body.get("partial_images").and_then(Value::as_u64).unwrap_or(0),
    })
}
