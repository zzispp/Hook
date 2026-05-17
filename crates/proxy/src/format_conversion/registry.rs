use serde_json::Value;

use super::{
    ApiFormat, FormatConversionError, StreamConversionState,
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
        let internal = self.normalizer(source).request_to_internal(request)?;
        self.normalizer(target).request_from_internal(&internal)
    }

    pub fn convert_response(&self, response: &Value, source: ApiFormat, target: ApiFormat) -> Result<Value, FormatConversionError> {
        if source == target {
            return Ok(response.clone());
        }
        let internal = self.normalizer(source).response_to_internal(response)?;
        self.normalizer(target).response_from_internal(&internal)
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
