use proxy::format_conversion::ApiFormat;
use serde_json::Value;
use types::provider::RoutingRequestFeatures;

use crate::llm_proxy::{LlmProxyError, formats};

use super::stream_transport::token_estimator::estimate_text_tokens;

pub(super) fn routing_request_features(
    api_format: &str,
    body: &Value,
    model: &str,
    is_stream: bool,
    required_capability: Option<&str>,
) -> Result<RoutingRequestFeatures, LlmProxyError> {
    let metadata = formats::endpoint_metadata(api_format, is_stream)?;
    let input = input_token_estimate(metadata.data_format, body, model);
    let output = output_token_estimate(metadata.data_format, body);
    Ok(RoutingRequestFeatures::new(api_format, is_stream, input, output, required_capability))
}

fn input_token_estimate(format: ApiFormat, body: &Value, model: &str) -> Option<u64> {
    if !supports_input_estimate(format) {
        return None;
    }
    let mut text = String::new();
    collect_input_fields(format, body, &mut text);
    Some(u64::try_from(estimate_text_tokens(model_name(body).unwrap_or(model), &text).max(0)).unwrap_or(u64::MAX))
}

fn output_token_estimate(format: ApiFormat, body: &Value) -> Option<u64> {
    match format {
        ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact => numeric_field(body, "max_output_tokens"),
        ApiFormat::OpenAiChat => numeric_field(body, "max_completion_tokens").or_else(|| numeric_field(body, "max_tokens")),
        ApiFormat::ClaudeChat => numeric_field(body, "max_tokens"),
        ApiFormat::GeminiChat => gemini_max_output_tokens(body),
        _ => None,
    }
}

fn collect_input_fields(format: ApiFormat, body: &Value, output: &mut String) {
    match format {
        ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact => {
            collect_named_fields(body, &["input", "instructions", "metadata", "text", "tool_choice", "prompt", "tools"], output);
        }
        ApiFormat::OpenAiChat => collect_named_fields(body, &["messages", "tools", "prompt", "input"], output),
        ApiFormat::ClaudeChat => collect_named_fields(body, &["messages", "system", "tools", "tool_choice"], output),
        ApiFormat::GeminiChat => collect_named_fields(body, &["contents", "systemInstruction", "tools", "toolConfig"], output),
        _ => {}
    }
}

fn collect_named_fields(body: &Value, fields: &[&str], output: &mut String) {
    let Some(object) = body.as_object() else {
        return;
    };
    for field in fields {
        collect_value_text(object.get(*field), output);
    }
}

fn collect_value_text(value: Option<&Value>, output: &mut String) {
    match value {
        Some(Value::String(text)) => {
            output.push(' ');
            output.push_str(text);
        }
        Some(Value::Array(items)) => {
            for item in items {
                collect_value_text(Some(item), output);
            }
        }
        Some(Value::Object(object)) => {
            for value in object.values() {
                collect_value_text(Some(value), output);
            }
        }
        Some(Value::Number(number)) => {
            output.push(' ');
            output.push_str(&number.to_string());
        }
        Some(Value::Bool(value)) => {
            output.push(' ');
            output.push_str(if *value { "true" } else { "false" });
        }
        Some(Value::Null) | None => {}
    }
}

fn supports_input_estimate(format: ApiFormat) -> bool {
    matches!(
        format,
        ApiFormat::OpenAiChat | ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact | ApiFormat::ClaudeChat | ApiFormat::GeminiChat
    )
}

fn numeric_field(body: &Value, key: &str) -> Option<u64> {
    body.get(key).and_then(Value::as_u64)
}

fn gemini_max_output_tokens(body: &Value) -> Option<u64> {
    body.get("generationConfig")
        .or_else(|| body.get("generation_config"))
        .and_then(|value| value.get("maxOutputTokens").or_else(|| value.get("max_output_tokens")))
        .and_then(Value::as_u64)
}

fn model_name(body: &Value) -> Option<&str> {
    body.get("model").and_then(Value::as_str).filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use types::provider::RoutingRequestSizeBucket;

    use super::routing_request_features;

    #[test]
    fn openai_chat_features_include_input_and_output_estimates() {
        let body = json!({
            "model": "gpt-5.5",
            "messages": [{"role": "user", "content": "hello world"}],
            "max_completion_tokens": 42,
            "stream": true
        });

        let features = routing_request_features("openai:chat", &body, "gpt-5.5", true, None).unwrap();

        assert_eq!(features.client_api_format, "openai:chat");
        assert!(features.is_stream);
        assert_eq!(features.output_token_estimate, Some(42));
        assert!(features.input_token_estimate.unwrap_or_default() > 0);
        assert_eq!(features.request_size_bucket, RoutingRequestSizeBucket::Tiny);
    }

    #[test]
    fn gemini_features_read_generation_config_output_limit() {
        let body = json!({
            "model": "gemini-test",
            "contents": [{"parts": [{"text": "hello"}]}],
            "generationConfig": {"maxOutputTokens": 256}
        });

        let features = routing_request_features("gemini:chat", &body, "gemini-test", false, Some("image_generation")).unwrap();

        assert_eq!(features.output_token_estimate, Some(256));
        assert_eq!(features.required_capability.as_deref(), Some("image_generation"));
    }
}
