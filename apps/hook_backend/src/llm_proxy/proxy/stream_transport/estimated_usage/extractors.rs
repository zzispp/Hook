use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use crate::llm_proxy::proxy::stream_transport::token_estimator::estimate_text_tokens;

const GEMINI_IMAGE_OUTPUT_TOKENS: i64 = 1400;

#[derive(Default)]
pub(super) struct OutputDelta {
    pub(super) text: String,
    pub(super) image_tokens: i64,
}

pub(super) fn supports_estimation(format: ApiFormat) -> bool {
    matches!(
        format,
        ApiFormat::OpenAiChat | ApiFormat::OpenAiResponses | ApiFormat::ClaudeChat | ApiFormat::GeminiChat
    )
}

pub(super) fn usage_semantic(format: ApiFormat) -> &'static str {
    match format {
        ApiFormat::OpenAiResponses => "responses",
        ApiFormat::ClaudeChat => "anthropic",
        ApiFormat::GeminiChat => "gemini",
        _ => "openai",
    }
}

pub(super) fn output_delta(format: ApiFormat, chunk: &Value, gemini_previous_output: &mut String) -> Option<OutputDelta> {
    match format {
        ApiFormat::OpenAiResponses => responses_delta(chunk),
        ApiFormat::OpenAiChat => chat_delta(chunk),
        ApiFormat::ClaudeChat => claude_delta(chunk),
        ApiFormat::GeminiChat => gemini_delta(chunk, gemini_previous_output),
        _ => None,
    }
}

pub(super) fn estimate_request_tokens(format: ApiFormat, request: &Value, model: &str) -> i64 {
    let mut text = String::new();
    match format {
        ApiFormat::OpenAiResponses => collect_request_fields(
            request,
            &["input", "instructions", "metadata", "text", "tool_choice", "prompt", "tools"],
            &mut text,
        ),
        ApiFormat::OpenAiChat => collect_request_fields(request, &["messages", "tools", "prompt", "input"], &mut text),
        ApiFormat::ClaudeChat => collect_request_fields(request, &["messages", "system", "tools", "tool_choice"], &mut text),
        ApiFormat::GeminiChat => collect_request_fields(request, &["contents", "systemInstruction", "tools", "toolConfig"], &mut text),
        _ => {}
    }
    estimate_text_tokens(model_name(request).unwrap_or(model), &text)
}

fn responses_delta(chunk: &Value) -> Option<OutputDelta> {
    match chunk.get("type").and_then(Value::as_str) {
        Some("response.output_text.delta" | "response.reasoning_summary_text.delta" | "response.function_call_arguments.delta") => {
            text_delta(chunk.get("delta").and_then(Value::as_str))
        }
        Some("response.output_item.added" | "response.output_item.done") => text_delta(function_call_item_text(chunk.get("item"))),
        _ => None,
    }
}

fn chat_delta(chunk: &Value) -> Option<OutputDelta> {
    let delta = chunk.get("choices")?.as_array()?.first()?.get("delta")?;
    text_delta(
        delta
            .get("content")
            .and_then(Value::as_str)
            .or_else(|| delta.get("reasoning_content").and_then(Value::as_str))
            .or_else(|| delta.get("tool_calls").and_then(tool_calls_text)),
    )
}

fn claude_delta(chunk: &Value) -> Option<OutputDelta> {
    match chunk.get("type").and_then(Value::as_str) {
        Some("content_block_start") => claude_content_block_start_delta(chunk),
        Some("content_block_delta") => claude_content_block_delta(chunk),
        _ => None,
    }
}

fn gemini_delta(chunk: &Value, previous_output: &mut String) -> Option<OutputDelta> {
    let candidates = chunk.get("candidates")?.as_array()?;
    let mut current_text = String::new();
    let mut image_tokens = 0;
    for candidate in candidates {
        let Some(parts) = candidate.get("content").and_then(|content| content.get("parts")).and_then(Value::as_array) else {
            continue;
        };
        image_tokens += gemini_image_tokens(parts);
        append_gemini_text_parts(parts, &mut current_text);
    }
    let delta = gemini_text_delta(&current_text, previous_output).to_owned();
    *previous_output = current_text;
    output_delta_value(&delta, image_tokens)
}

fn claude_content_block_start_delta(chunk: &Value) -> Option<OutputDelta> {
    let block = chunk.get("content_block")?;
    text_delta(block.get("text").and_then(Value::as_str).or_else(|| block.get("name").and_then(Value::as_str)))
}

fn claude_content_block_delta(chunk: &Value) -> Option<OutputDelta> {
    let delta = chunk.get("delta")?;
    text_delta(
        delta
            .get("text")
            .and_then(Value::as_str)
            .or_else(|| delta.get("thinking").and_then(Value::as_str))
            .or_else(|| delta.get("partial_json").and_then(Value::as_str)),
    )
}

fn function_call_item_text(item: Option<&Value>) -> Option<&str> {
    let item = item?;
    if item.get("type").and_then(Value::as_str) != Some("function_call") {
        return None;
    }
    item.get("arguments")
        .and_then(Value::as_str)
        .or_else(|| item.get("arguments_json").and_then(Value::as_str))
        .or_else(|| item.get("name").and_then(Value::as_str))
}

fn tool_calls_text(value: &Value) -> Option<&str> {
    value.as_array()?.iter().find_map(|item| item.get("function")).and_then(|function| {
        function
            .get("arguments")
            .and_then(Value::as_str)
            .or_else(|| function.get("name").and_then(Value::as_str))
    })
}

fn append_gemini_text_parts(parts: &[Value], output: &mut String) {
    for part in parts {
        if let Some(text) = part.get("text").and_then(Value::as_str) {
            output.push_str(text);
        }
        append_json_text(part.get("functionCall").or_else(|| part.get("function_call")), output);
    }
}

fn gemini_image_tokens(parts: &[Value]) -> i64 {
    let count = parts
        .iter()
        .filter(|part| part.get("inlineData").or_else(|| part.get("inline_data")).is_some())
        .count();
    i64::try_from(count).unwrap_or(i64::MAX) * GEMINI_IMAGE_OUTPUT_TOKENS
}

fn gemini_text_delta<'a>(current: &'a str, previous: &str) -> &'a str {
    current.strip_prefix(previous).unwrap_or(current)
}

fn text_delta(value: Option<&str>) -> Option<OutputDelta> {
    value.map(|text| OutputDelta {
        text: text.to_owned(),
        image_tokens: 0,
    })
}

fn output_delta_value(text: &str, image_tokens: i64) -> Option<OutputDelta> {
    if text.is_empty() && image_tokens == 0 {
        return None;
    }
    Some(OutputDelta {
        text: text.to_owned(),
        image_tokens,
    })
}

fn model_name(request: &Value) -> Option<&str> {
    request.get("model").and_then(Value::as_str)
}

fn collect_request_fields(request: &Value, keys: &[&str], output: &mut String) {
    for key in keys {
        append_json_text(request.get(*key), output);
    }
}

fn append_json_text(value: Option<&Value>, output: &mut String) {
    match value {
        Some(Value::String(text)) => append_piece(output, text),
        Some(Value::Array(items)) => {
            for item in items {
                append_json_text(Some(item), output);
            }
        }
        Some(Value::Object(map)) => append_object_text(map, output),
        _ => {}
    }
}

fn append_object_text(map: &serde_json::Map<String, Value>, output: &mut String) {
    for key in [
        "role",
        "text",
        "content",
        "parts",
        "input",
        "output",
        "name",
        "description",
        "arguments",
        "args",
        "call_id",
        "tool_calls",
        "tools",
        "function",
        "functionCall",
        "parameters",
    ] {
        append_json_text(map.get(key), output);
    }
}

fn append_piece(output: &mut String, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(text);
}
