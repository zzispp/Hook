use std::collections::VecDeque;

use serde_json::Value;

use super::{
    ApiFormat, FormatConversionError, InternalContentBlock, InternalError, InternalRequest, StreamConversionState,
    normalizer::FormatNormalizer,
    normalizers::{ClaudeNormalizer, GeminiNormalizer, OpenAiNormalizer, OpenAiResponsesNormalizer},
};

pub struct FormatConversionRegistry {
    openai: OpenAiNormalizer,
    openai_responses: OpenAiResponsesNormalizer,
    gemini: GeminiNormalizer,
    claude: ClaudeNormalizer,
}

pub struct StreamChunkConversion<'a> {
    pub chunk: &'a Value,
    pub source: ApiFormat,
    pub target: ApiFormat,
    pub state: &'a mut StreamConversionState,
}

impl Default for FormatConversionRegistry {
    fn default() -> Self {
        Self {
            openai: OpenAiNormalizer,
            openai_responses: OpenAiResponsesNormalizer,
            gemini: GeminiNormalizer,
            claude: ClaudeNormalizer,
        }
    }
}

impl FormatConversionRegistry {
    pub fn convert_request(&self, request: &Value, source: ApiFormat, target: ApiFormat) -> Result<Value, FormatConversionError> {
        if source == target {
            return Ok(request.clone());
        }
        let mut internal = self.normalizer(source).request_to_internal(request)?;
        repair_internal_tool_call_ids(&mut internal);
        self.normalizer(target).request_from_internal(&internal)
    }

    pub fn convert_response(&self, response: &Value, source: ApiFormat, target: ApiFormat) -> Result<Value, FormatConversionError> {
        if source == target {
            return Ok(response.clone());
        }
        let internal = self.normalizer(source).response_to_internal(response)?;
        self.normalizer(target).response_from_internal(&internal)
    }

    pub fn convert_error(&self, error: &Value, status: Option<u16>, source: ApiFormat, target: ApiFormat) -> Result<Value, FormatConversionError> {
        if source == target {
            return Ok(error.clone());
        }
        let internal = self.normalizer(source).error_to_internal(error, status)?;
        self.normalizer(target).error_from_internal(&internal)
    }

    pub fn convert_stream(&self, chunks: &[Value], source: ApiFormat, target: ApiFormat) -> Result<Vec<Value>, FormatConversionError> {
        if source == target {
            return Ok(chunks.to_vec());
        }
        let internal = self.normalizer(source).stream_to_internal(chunks)?;
        self.normalizer(target).stream_from_internal(&internal)
    }

    pub fn convert_stream_chunk(&self, input: StreamChunkConversion<'_>) -> Result<Vec<Value>, FormatConversionError> {
        if input.source == input.target {
            return Ok(vec![input.chunk.clone()]);
        }
        let events = self.normalizer(input.source).stream_chunk_to_internal(input.chunk, input.state)?;
        self.convert_stream_events(events, input.target, input.state)
    }

    pub fn flush_stream(&self, source: ApiFormat, target: ApiFormat, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
        if source == target {
            return Ok(Vec::new());
        }
        let events = self.normalizer(source).stream_flush_to_internal(state)?;
        self.convert_stream_events(events, target, state)
    }

    fn convert_stream_events(
        &self,
        events: Vec<super::InternalStreamEvent>,
        target: ApiFormat,
        state: &mut StreamConversionState,
    ) -> Result<Vec<Value>, FormatConversionError> {
        let mut converted = Vec::new();
        for event in events {
            converted.extend(self.normalizer(target).stream_event_from_internal(&event, state)?);
        }
        Ok(converted)
    }

    pub fn can_convert(&self, source: ApiFormat, target: ApiFormat, require_stream: bool) -> bool {
        if !source.supports_chat_conversion() || !target.supports_chat_conversion() {
            return false;
        }
        let request = source != target;
        let response = source != target;
        if require_stream { request && response } else { request || source == target }
    }

    fn normalizer(&self, format: ApiFormat) -> &dyn FormatNormalizer {
        match format {
            ApiFormat::OpenAiChat => &self.openai,
            ApiFormat::OpenAiResponses => &self.openai_responses,
            ApiFormat::GeminiChat => &self.gemini,
            ApiFormat::ClaudeChat => &self.claude,
            _ => unreachable!("non-chat API formats do not have format normalizers"),
        }
    }
}

fn repair_internal_tool_call_ids(internal: &mut InternalRequest) {
    let mut pending_tool_ids = VecDeque::new();
    let mut auto_counter = 0_u32;
    for message in &mut internal.messages {
        repair_message_tool_ids(&mut message.content, &mut pending_tool_ids, &mut auto_counter);
    }
}

fn repair_message_tool_ids(blocks: &mut [InternalContentBlock], pending_tool_ids: &mut VecDeque<String>, auto_counter: &mut u32) {
    for block in blocks {
        match block {
            InternalContentBlock::ToolUse { id, .. } => {
                *id = repaired_tool_id(id, auto_counter);
                pending_tool_ids.push_back(id.clone());
            }
            InternalContentBlock::ToolResult { tool_use_id, .. } => {
                *tool_use_id = repaired_tool_result_id(tool_use_id, pending_tool_ids, auto_counter);
            }
            _ => {}
        }
    }
}

fn repaired_tool_id(id: &str, auto_counter: &mut u32) -> String {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return next_tool_id(auto_counter);
    }
    trimmed.to_owned()
}

fn repaired_tool_result_id(tool_use_id: &str, pending_tool_ids: &mut VecDeque<String>, auto_counter: &mut u32) -> String {
    let trimmed = tool_use_id.trim();
    if !trimmed.is_empty() {
        remove_pending_tool_id(pending_tool_ids, trimmed);
        return trimmed.to_owned();
    }
    pending_tool_ids.pop_front().unwrap_or_else(|| next_tool_id(auto_counter))
}

fn remove_pending_tool_id(pending_tool_ids: &mut VecDeque<String>, id: &str) {
    if let Some(index) = pending_tool_ids.iter().position(|item| item == id) {
        pending_tool_ids.remove(index);
    }
}

fn next_tool_id(auto_counter: &mut u32) -> String {
    *auto_counter = auto_counter.saturating_add(1);
    format!("call_auto_{auto_counter}")
}

impl From<InternalError> for Value {
    fn from(error: InternalError) -> Self {
        serde_json::json!({
            "error": {
                "message": error.message,
                "type": error.error_type,
                "code": error.code,
                "param": error.param,
            }
        })
    }
}
