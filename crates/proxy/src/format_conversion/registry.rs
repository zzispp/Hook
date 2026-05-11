use serde_json::Value;

use super::{
    ApiFormat, FormatConversionError,
    normalizer::FormatNormalizer,
    normalizers::{ClaudeNormalizer, GeminiNormalizer, OpenAiNormalizer},
};

pub struct FormatConversionRegistry {
    openai: OpenAiNormalizer,
    gemini: GeminiNormalizer,
    claude: ClaudeNormalizer,
}

impl Default for FormatConversionRegistry {
    fn default() -> Self {
        Self {
            openai: OpenAiNormalizer,
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

    pub fn can_convert(&self, source: ApiFormat, target: ApiFormat, require_stream: bool) -> bool {
        let request = source != target;
        let response = source != target;
        if require_stream { request && response } else { request || source == target }
    }

    fn normalizer(&self, format: ApiFormat) -> &dyn FormatNormalizer {
        match format {
            ApiFormat::OpenAiChat => &self.openai,
            ApiFormat::GeminiChat => &self.gemini,
            ApiFormat::ClaudeChat => &self.claude,
        }
    }
}
