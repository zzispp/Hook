use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use crate::llm_proxy::{LlmProxyError, audit::TokenUsage, proxy::usage};

pub(super) struct StreamUsageParser {
    format: ApiFormat,
    buffer: Vec<u8>,
}

impl StreamUsageParser {
    pub(super) fn new(format: ApiFormat) -> Self {
        Self { format, buffer: Vec::new() }
    }

    pub(super) fn consume(&mut self, bytes: &[u8]) -> Result<Option<TokenUsage>, LlmProxyError> {
        self.buffer.extend_from_slice(bytes);
        let mut collected = None;
        while let Some(line) = self.next_line() {
            if let Some(incoming) = self.usage_from_line(line.as_slice())? {
                collected = usage::merge(collected, incoming);
            }
        }
        Ok(collected)
    }

    pub(super) fn finish(&mut self) -> Result<Option<TokenUsage>, LlmProxyError> {
        if self.buffer.is_empty() {
            return Ok(None);
        }
        let line = std::mem::take(&mut self.buffer);
        self.usage_from_line(line.as_slice())
    }

    fn next_line(&mut self) -> Option<Vec<u8>> {
        let position = self.buffer.iter().position(|byte| *byte == b'\n')?;
        Some(self.buffer.drain(..=position).collect())
    }

    fn usage_from_line(&self, line: &[u8]) -> Result<Option<TokenUsage>, LlmProxyError> {
        let line = std::str::from_utf8(line).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        let Some(payload) = line.trim_end_matches(['\r', '\n']).strip_prefix("data:") else {
            return Ok(None);
        };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            return Ok(None);
        }
        let chunk = serde_json::from_str::<Value>(payload).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        Ok(usage::from_stream_chunk(&chunk, self.format))
    }
}

#[cfg(test)]
mod tests {
    use proxy::format_conversion::ApiFormat;

    use super::StreamUsageParser;

    #[test]
    fn extracts_openai_chat_usage_across_network_chunks() {
        let mut parser = StreamUsageParser::new(ApiFormat::OpenAiChat);

        let first = br#"data: {"choices":[],"usage":{"prompt_tokens":12"#;
        let second = b",\"completion_tokens\":8,\"total_tokens\":20}}\n\n";

        assert!(parser.consume(first).unwrap().is_none());
        let usage = parser.consume(second).unwrap().expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(12));
        assert_eq!(usage.completion_tokens, Some(8));
        assert_eq!(usage.total_tokens, Some(20));
    }

    #[test]
    fn finish_extracts_usage_without_trailing_newline() {
        let mut parser = StreamUsageParser::new(ApiFormat::OpenAiChat);
        parser
            .consume(br#"data: {"choices":[],"usage":{"prompt_tokens":3,"completion_tokens":4}}"#)
            .unwrap();

        let usage = parser.finish().unwrap().expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(3));
        assert_eq!(usage.completion_tokens, Some(4));
        assert_eq!(usage.total_tokens, Some(7));
    }
}
