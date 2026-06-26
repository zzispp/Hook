use proxy::format_conversion::ApiFormat;
use serde_json::Value;

pub(super) struct StreamOutputStartDetector {
    format: ApiFormat,
    buffer: Vec<u8>,
    gemini_previous_output: String,
}

impl StreamOutputStartDetector {
    pub(super) fn new(format: ApiFormat) -> Self {
        Self {
            format,
            buffer: Vec::new(),
            gemini_previous_output: String::new(),
        }
    }

    pub(super) fn consume(&mut self, bytes: &[u8]) -> Result<bool, serde_json::Error> {
        self.buffer.extend_from_slice(bytes);
        while let Some(line) = self.next_line() {
            if self.line_starts_output(&line)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn finish(&mut self) -> Result<bool, serde_json::Error> {
        if self.buffer.is_empty() {
            return Ok(false);
        }
        let line = std::mem::take(&mut self.buffer);
        self.line_starts_output(&line)
    }

    fn next_line(&mut self) -> Option<Vec<u8>> {
        let position = self.buffer.iter().position(|byte| *byte == b'\n')?;
        Some(self.buffer.drain(..=position).collect())
    }

    fn line_starts_output(&mut self, line: &[u8]) -> Result<bool, serde_json::Error> {
        let Ok(line) = std::str::from_utf8(line) else {
            return Ok(false);
        };
        let Some(payload) = line.trim_end_matches(['\r', '\n']).strip_prefix("data:") else {
            return Ok(false);
        };
        output_payload_starts(self.format, payload.trim(), &mut self.gemini_previous_output)
    }
}

fn output_payload_starts(format: ApiFormat, payload: &str, gemini_previous_output: &mut String) -> Result<bool, serde_json::Error> {
    if payload.is_empty() || payload == "[DONE]" {
        return Ok(false);
    }
    let chunk = serde_json::from_str::<Value>(payload)?;
    Ok(chunk_starts_output(format, &chunk, gemini_previous_output))
}

fn chunk_starts_output(format: ApiFormat, chunk: &Value, gemini_previous_output: &mut String) -> bool {
    match format {
        ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact => responses_starts_output(chunk),
        ApiFormat::OpenAiChat | ApiFormat::OpenAiCompletion => openai_starts_output(chunk),
        ApiFormat::ClaudeChat => claude_starts_output(chunk),
        ApiFormat::GeminiChat => gemini_starts_output(chunk, gemini_previous_output),
        _ => true,
    }
}

fn responses_starts_output(chunk: &Value) -> bool {
    match chunk.get("type").and_then(Value::as_str) {
        Some("response.output_text.delta" | "response.reasoning_summary_text.delta" | "response.function_call_arguments.delta") => {
            chunk.get("delta").and_then(non_empty_str).is_some()
        }
        Some("response.content_part.added" | "response.content_part.done") => {
            content_part_text(chunk.get("part")).is_some() || content_part_refusal(chunk.get("part")).is_some()
        }
        Some("response.reasoning_summary_text.done") => {
            chunk.get("text").and_then(non_empty_str).is_some()
                || chunk
                    .get("part")
                    .and_then(Value::as_object)
                    .and_then(|part| part.get("text"))
                    .and_then(non_empty_str)
                    .is_some()
        }
        Some("response.output_item.added" | "response.output_item.done" | "response.function_call_arguments.done") => {
            function_call_item_text(chunk.get("item")).is_some()
                || reasoning_item_text(chunk.get("item")).is_some()
                || message_item_text(chunk.get("item")).is_some()
        }
        _ => false,
    }
}

fn openai_starts_output(chunk: &Value) -> bool {
    chunk
        .get("choices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(choice_delta_starts_output)
}

fn claude_starts_output(chunk: &Value) -> bool {
    match chunk.get("type").and_then(Value::as_str) {
        Some("content_block_start") => claude_start_text(chunk.get("content_block")).is_some(),
        Some("content_block_delta") => claude_delta_text(chunk.get("delta")).is_some(),
        _ => false,
    }
}

fn gemini_starts_output(chunk: &Value, previous_output: &mut String) -> bool {
    let mut current_text = String::new();
    let mut has_binary_output = false;
    for part in gemini_parts(chunk) {
        append_gemini_text(part, &mut current_text);
        has_binary_output |= part.get("inlineData").or_else(|| part.get("inline_data")).is_some();
    }
    let has_text_delta = !current_text.strip_prefix(previous_output.as_str()).unwrap_or(&current_text).is_empty();
    *previous_output = current_text;
    has_text_delta || has_binary_output
}

fn choice_delta_starts_output(choice: &Value) -> bool {
    let Some(delta) = choice.get("delta") else {
        return false;
    };
    delta.get("content").and_then(non_empty_str).is_some()
        || delta.get("reasoning_content").and_then(non_empty_str).is_some()
        || delta.get("tool_calls").and_then(tool_calls_text).is_some()
}

fn function_call_item_text(item: Option<&Value>) -> Option<&str> {
    let item = item?;
    if item.get("type").and_then(Value::as_str) != Some("function_call") {
        return None;
    }
    item.get("arguments")
        .and_then(non_empty_str)
        .or_else(|| item.get("arguments_json").and_then(non_empty_str))
        .or_else(|| item.get("name").and_then(non_empty_str))
}

fn message_item_text(item: Option<&Value>) -> Option<&str> {
    let item = item?;
    if item.get("type").and_then(Value::as_str) != Some("message") {
        return None;
    }
    item.get("content").and_then(Value::as_array)?.iter().find_map(|part| {
        let part = part.as_object()?;
        if matches!(part.get("type").and_then(Value::as_str), Some("output_text" | "text")) {
            return part.get("text").and_then(non_empty_str);
        }
        part.get("refusal").and_then(non_empty_str)
    })
}

fn reasoning_item_text(item: Option<&Value>) -> Option<&str> {
    let item = item?;
    if item.get("type").and_then(Value::as_str) != Some("reasoning") {
        return None;
    }
    item.get("summary").and_then(Value::as_array)?.iter().find_map(|part| {
        let part = part.as_object()?;
        if part.get("type").and_then(Value::as_str) != Some("summary_text") {
            return None;
        }
        part.get("text").and_then(non_empty_str)
    })
}

fn content_part_text(part: Option<&Value>) -> Option<&str> {
    let part = part?.as_object()?;
    if !matches!(part.get("type").and_then(Value::as_str), Some("output_text" | "text")) {
        return None;
    }
    part.get("text").and_then(non_empty_str)
}

fn content_part_refusal(part: Option<&Value>) -> Option<&str> {
    let part = part?.as_object()?;
    part.get("refusal").and_then(non_empty_str)
}

fn tool_calls_text(value: &Value) -> Option<&str> {
    value.as_array()?.iter().find_map(|item| item.get("function")).and_then(|function| {
        function
            .get("arguments")
            .and_then(non_empty_str)
            .or_else(|| function.get("name").and_then(non_empty_str))
    })
}

fn claude_start_text(block: Option<&Value>) -> Option<&str> {
    let block = block?;
    block.get("text").and_then(non_empty_str).or_else(|| block.get("name").and_then(non_empty_str))
}

fn claude_delta_text(delta: Option<&Value>) -> Option<&str> {
    let delta = delta?;
    delta
        .get("text")
        .and_then(non_empty_str)
        .or_else(|| delta.get("thinking").and_then(non_empty_str))
        .or_else(|| delta.get("partial_json").and_then(non_empty_str))
}

fn non_empty_str(value: &Value) -> Option<&str> {
    value.as_str().filter(|text| !text.is_empty())
}

fn gemini_parts(chunk: &Value) -> impl Iterator<Item = &Value> {
    chunk
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|candidate| candidate.get("content").and_then(|content| content.get("parts")).and_then(Value::as_array))
        .flatten()
}

fn append_gemini_text(part: &Value, output: &mut String) {
    if let Some(text) = part.get("text").and_then(Value::as_str) {
        output.push_str(text);
    }
    append_json_text(part.get("functionCall").or_else(|| part.get("function_call")), output);
}

fn append_json_text(value: Option<&Value>, output: &mut String) {
    match value {
        Some(Value::String(text)) => output.push_str(text),
        Some(Value::Array(items)) => {
            for item in items {
                append_json_text(Some(item), output);
            }
        }
        Some(Value::Object(map)) => {
            for value in map.values() {
                append_json_text(Some(value), output);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
