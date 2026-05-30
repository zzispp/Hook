use serde_json::{Value, json};

use super::{ApiFormat, FormatConversionError, StreamConversionState};

#[derive(Default)]
pub struct FormatConversionRegistry;

pub struct StreamChunkConversion<'a> {
    pub chunk: &'a Value,
    pub source: ApiFormat,
    pub target: ApiFormat,
    pub state: &'a mut StreamConversionState,
}

impl FormatConversionRegistry {
    pub fn convert_request(&self, request: &Value, source: ApiFormat, target: ApiFormat) -> Result<Value, FormatConversionError> {
        if source == target {
            return Ok(request.clone());
        }
        reject_unknown_openai_responses_request_items(request, source)?;
        formats::convert_request(source.as_format_id()?, target.as_format_id()?, request, &formats::FormatContext::default()).map_err(Into::into)
    }

    pub fn convert_response(&self, response: &Value, source: ApiFormat, target: ApiFormat) -> Result<Value, FormatConversionError> {
        if source == target {
            return Ok(response.clone());
        }
        reject_unknown_openai_responses_response_items(response, source)?;
        formats::convert_response(source.as_format_id()?, target.as_format_id()?, response, &formats::FormatContext::default()).map_err(Into::into)
    }

    pub fn convert_error(&self, error: &Value, status: Option<u16>, source: ApiFormat, target: ApiFormat) -> Result<Value, FormatConversionError> {
        if source == target {
            return Ok(error.clone());
        }
        let (message, code, kind) = parse_error(error, status, source);
        formats::api::build_core_error_body_for_client_format(target.as_format_id()?, &message, code.as_deref(), kind)
            .ok_or_else(|| FormatConversionError::unsupported_feature("error", format!("target format {target:?}")))
    }

    pub fn convert_stream(&self, chunks: &[Value], source: ApiFormat, target: ApiFormat) -> Result<Vec<Value>, FormatConversionError> {
        let mut state = StreamConversionState::default();
        let mut converted = Vec::new();
        for chunk in chunks {
            converted.extend(self.convert_stream_chunk(StreamChunkConversion {
                chunk,
                source,
                target,
                state: &mut state,
            })?);
        }
        converted.extend(self.flush_stream(source, target, &mut state)?);
        Ok(converted)
    }

    pub fn convert_stream_chunk(&self, input: StreamChunkConversion<'_>) -> Result<Vec<Value>, FormatConversionError> {
        if input.source == input.target {
            return Ok(vec![input.chunk.clone()]);
        }
        ensure_stream_supported(input.source, input.target)?;
        reject_unknown_openai_responses_stream_chunk(input.chunk, input.source)?;
        let report_context = stream_report_context(input.source, input.target);
        let bytes = input
            .state
            .matrix
            .transform_line(&report_context, encode_data_line(input.chunk)?)
            .map_err(stream_error)?;
        decode_sse_values(&bytes)
    }

    pub fn flush_stream(&self, source: ApiFormat, target: ApiFormat, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
        if source == target {
            return Ok(Vec::new());
        }
        ensure_stream_supported(source, target)?;
        let bytes = state.matrix.finish(&stream_report_context(source, target)).map_err(stream_error)?;
        decode_sse_values(&bytes)
    }

    pub fn can_convert(&self, source: ApiFormat, target: ApiFormat, require_stream: bool) -> bool {
        if source == target {
            return true;
        }
        if require_stream {
            return source.is_stream_convertible() && target.is_stream_convertible();
        }
        if source == ApiFormat::OpenAiChat && target == ApiFormat::OpenAiResponsesCompact {
            return true;
        }
        let Ok(source_id) = source.as_format_id() else {
            return false;
        };
        let Ok(target_id) = target.as_format_id() else {
            return false;
        };
        formats::request_candidate_api_format_preference(source_id, target_id).is_some()
    }
}

fn ensure_stream_supported(source: ApiFormat, target: ApiFormat) -> Result<(), FormatConversionError> {
    if source.is_stream_convertible() && target.is_stream_convertible() {
        return Ok(());
    }
    Err(FormatConversionError::unsupported_feature(
        "stream",
        format!("stream conversion from {source:?} to {target:?}"),
    ))
}

fn stream_report_context(source: ApiFormat, target: ApiFormat) -> Value {
    json!({
        "provider_api_format": source.as_format_id().unwrap_or_default(),
        "client_api_format": target.as_format_id().unwrap_or_default(),
    })
}

fn encode_data_line(chunk: &Value) -> Result<Vec<u8>, FormatConversionError> {
    let mut bytes = b"data: ".to_vec();
    bytes.extend(serde_json::to_vec(chunk).map_err(|error| FormatConversionError::invalid_payload("stream", error.to_string()))?);
    bytes.push(b'\n');
    Ok(bytes)
}

fn decode_sse_values(bytes: &[u8]) -> Result<Vec<Value>, FormatConversionError> {
    std::str::from_utf8(bytes)
        .map_err(|error| FormatConversionError::invalid_payload("stream", error.to_string()))?
        .split("\n\n")
        .filter(|event| !event.trim().is_empty())
        .filter_map(decode_sse_event)
        .collect()
}

fn decode_sse_event(event: &str) -> Option<Result<Value, FormatConversionError>> {
    let data = event.lines().find_map(|line| line.strip_prefix("data:"))?.trim();
    if data.is_empty() || data == "[DONE]" {
        return None;
    }
    Some(serde_json::from_str(data).map_err(|error| FormatConversionError::invalid_payload("stream", error.to_string())))
}

fn stream_error(error: impl ToString) -> FormatConversionError {
    FormatConversionError::unsupported_content("stream", error.to_string())
}

fn parse_error(error: &Value, status: Option<u16>, source: ApiFormat) -> (String, Option<String>, formats::api::LocalCoreSyncErrorKind) {
    let message = error_message(error).unwrap_or_else(|| "upstream request failed".to_owned());
    let code = error_code(error);
    (message, code, error_kind(error, status, source))
}

fn error_message(error: &Value) -> Option<String> {
    error
        .pointer("/error/message")
        .or_else(|| error.get("message"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn error_code(error: &Value) -> Option<String> {
    error.pointer("/error/code").or_else(|| error.get("code")).and_then(|value| match value {
        Value::String(text) if !text.trim().is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    })
}

fn error_kind(error: &Value, status: Option<u16>, source: ApiFormat) -> formats::api::LocalCoreSyncErrorKind {
    if matches!(source, ApiFormat::GeminiChat) {
        return gemini_error_kind(error, status);
    }
    provider_error_kind(error_type(error).as_deref(), status)
}

fn error_type(error: &Value) -> Option<String> {
    error
        .pointer("/error/type")
        .or_else(|| error.get("type"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn gemini_error_kind(error: &Value, status: Option<u16>) -> formats::api::LocalCoreSyncErrorKind {
    provider_error_kind(error.pointer("/error/status").and_then(Value::as_str), status)
}

fn provider_error_kind(kind: Option<&str>, status: Option<u16>) -> formats::api::LocalCoreSyncErrorKind {
    use formats::api::LocalCoreSyncErrorKind;
    match (kind.unwrap_or_default(), status) {
        ("invalid_request_error" | "INVALID_ARGUMENT", _) | (_, Some(400)) => LocalCoreSyncErrorKind::InvalidRequest,
        ("authentication_error" | "UNAUTHENTICATED", _) | (_, Some(401)) => LocalCoreSyncErrorKind::Authentication,
        ("permission_error" | "PERMISSION_DENIED", _) | (_, Some(403)) => LocalCoreSyncErrorKind::PermissionDenied,
        ("not_found_error" | "NOT_FOUND", _) | (_, Some(404)) => LocalCoreSyncErrorKind::NotFound,
        ("rate_limit_error" | "RESOURCE_EXHAUSTED", _) | (_, Some(429)) => LocalCoreSyncErrorKind::RateLimit,
        ("context_length_exceeded", _) => LocalCoreSyncErrorKind::ContextLengthExceeded,
        ("overloaded_error" | "UNAVAILABLE", _) | (_, Some(503)) => LocalCoreSyncErrorKind::Overloaded,
        _ => LocalCoreSyncErrorKind::ServerError,
    }
}

fn reject_unknown_openai_responses_request_items(request: &Value, source: ApiFormat) -> Result<(), FormatConversionError> {
    if source != ApiFormat::OpenAiResponses {
        return Ok(());
    }
    reject_unknown_items(
        request.get("input"),
        &["message", "function_call", "web_search_call", "function_call_output"],
        "input",
    )
}

fn reject_unknown_openai_responses_response_items(response: &Value, source: ApiFormat) -> Result<(), FormatConversionError> {
    if source != ApiFormat::OpenAiResponses {
        return Ok(());
    }
    reject_unknown_items(
        response.get("output"),
        &[
            "message",
            "reasoning",
            "function_call",
            "web_search_call",
            "function_call_output",
            "image_generation_call",
            "output_text",
            "text",
            "output_image",
            "image_url",
            "file",
            "input_file",
            "input_audio",
        ],
        "output",
    )
}

fn reject_unknown_openai_responses_stream_chunk(chunk: &Value, source: ApiFormat) -> Result<(), FormatConversionError> {
    if source != ApiFormat::OpenAiResponses {
        return Ok(());
    }
    match chunk.get("type").and_then(Value::as_str).unwrap_or_default() {
        "response.output_item.added" | "response.output_item.done" => reject_unknown_openai_responses_stream_item(chunk),
        event if event.starts_with("response.custom_tool_") => Err(FormatConversionError::unsupported_feature(
            "openai:responses",
            format!("unsupported stream event type {event}"),
        )),
        _ => Ok(()),
    }
}

fn reject_unknown_openai_responses_stream_item(chunk: &Value) -> Result<(), FormatConversionError> {
    let Some(item) = chunk.get("item") else {
        return Ok(());
    };
    reject_unknown_items(
        Some(&Value::Array(vec![item.clone()])),
        &["message", "reasoning", "function_call", "function_call_output", "image_generation_call"],
        "output",
    )
}

fn reject_unknown_items(value: Option<&Value>, allowed: &[&str], field: &'static str) -> Result<(), FormatConversionError> {
    let Some(items) = value.and_then(Value::as_array) else {
        return Ok(());
    };
    for item in items {
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("message");
        if !allowed.contains(&item_type) {
            return Err(FormatConversionError::unsupported_feature(
                "openai:responses",
                format!("unsupported {field} item type {item_type}"),
            ));
        }
    }
    Ok(())
}
