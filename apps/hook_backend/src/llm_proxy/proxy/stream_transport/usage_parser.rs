use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use crate::llm_proxy::{
    LlmProxyError,
    audit::TokenUsage,
    proxy::{stream_transport::completion::response_id_from_chunk, usage},
};

#[derive(Default)]
pub(super) struct StreamParseResult {
    pub(super) usage: Option<TokenUsage>,
    pub(super) response_id: Option<String>,
    pub(super) completed: bool,
}

pub(super) struct StreamUsageParser {
    format: ApiFormat,
    buffer: Vec<u8>,
}

impl StreamUsageParser {
    pub(super) fn new(format: ApiFormat) -> Self {
        Self { format, buffer: Vec::new() }
    }

    pub(super) fn consume(&mut self, bytes: &[u8]) -> Result<StreamParseResult, LlmProxyError> {
        self.buffer.extend_from_slice(bytes);
        let mut result = StreamParseResult::default();
        while let Some(line) = self.next_line() {
            result.merge(self.parse_line(line.as_slice())?);
        }
        Ok(result)
    }

    pub(super) fn finish(&mut self) -> Result<StreamParseResult, LlmProxyError> {
        if self.buffer.is_empty() {
            return Ok(StreamParseResult::default());
        }
        let line = std::mem::take(&mut self.buffer);
        self.parse_line(line.as_slice())
    }

    fn next_line(&mut self) -> Option<Vec<u8>> {
        let position = self.buffer.iter().position(|byte| *byte == b'\n')?;
        Some(self.buffer.drain(..=position).collect())
    }

    fn parse_line(&self, line: &[u8]) -> Result<StreamParseResult, LlmProxyError> {
        let line = std::str::from_utf8(line).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        let Some(payload) = line.trim_end_matches(['\r', '\n']).strip_prefix("data:") else {
            return Ok(StreamParseResult::default());
        };
        let payload = payload.trim();
        if payload.is_empty() {
            return Ok(StreamParseResult::default());
        }
        if payload == "[DONE]" {
            return Ok(StreamParseResult {
                completed: true,
                ..StreamParseResult::default()
            });
        }
        let chunk = serde_json::from_str::<Value>(payload).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        Ok(StreamParseResult {
            usage: usage::from_stream_chunk(&chunk, self.format),
            response_id: response_id_from_chunk(&chunk, self.format),
            completed: is_completion_chunk(&chunk, self.format),
        })
    }
}

impl StreamParseResult {
    fn merge(&mut self, incoming: Self) {
        if let Some(usage) = incoming.usage {
            self.usage = usage::merge(self.usage, usage);
        }
        self.response_id = incoming.response_id.or_else(|| self.response_id.take());
        self.completed |= incoming.completed;
    }
}

fn is_completion_chunk(chunk: &Value, format: ApiFormat) -> bool {
    match format {
        ApiFormat::OpenAiResponses => chunk.get("type").and_then(Value::as_str) == Some("response.completed"),
        ApiFormat::ClaudeChat => chunk.get("type").and_then(Value::as_str) == Some("message_stop"),
        ApiFormat::GeminiChat => gemini_completed(chunk),
        _ => false,
    }
}

fn gemini_completed(chunk: &Value) -> bool {
    chunk
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|candidate| candidate.get("finishReason").and_then(Value::as_str))
        .any(|reason| reason != "FINISH_REASON_UNSPECIFIED")
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

        assert!(parser.consume(first).unwrap().usage.is_none());
        let usage = parser.consume(second).unwrap().usage.expect("usage should be extracted");

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

        let usage = parser.finish().unwrap().usage.expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(3));
        assert_eq!(usage.completion_tokens, Some(4));
        assert_eq!(usage.total_tokens, Some(7));
    }

    #[test]
    fn detects_openai_responses_completion_with_usage() {
        let mut parser = StreamUsageParser::new(ApiFormat::OpenAiResponses);
        let result = parser
            .consume(b"data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":12,\"output_tokens\":7,\"total_tokens\":19}}}\n\n")
            .unwrap();

        let usage = result.usage.expect("usage should be extracted");

        assert!(result.completed);
        assert_eq!(usage.prompt_tokens, Some(12));
        assert_eq!(usage.completion_tokens, Some(7));
        assert_eq!(usage.total_tokens, Some(19));
    }

    #[test]
    fn detects_done_marker_completion() {
        let mut parser = StreamUsageParser::new(ApiFormat::OpenAiChat);
        let result = parser.consume(b"data: [DONE]\n\n").unwrap();

        assert!(result.completed);
        assert!(result.usage.is_none());
    }

    #[test]
    fn detects_claude_message_stop_completion() {
        let mut parser = StreamUsageParser::new(ApiFormat::ClaudeChat);
        let result = parser.consume(b"data: {\"type\":\"message_stop\"}\n\n").unwrap();

        assert!(result.completed);
        assert!(result.usage.is_none());
    }

    #[test]
    fn detects_gemini_finish_reason_completion() {
        let mut parser = StreamUsageParser::new(ApiFormat::GeminiChat);
        let result = parser
            .consume(b"data: {\"candidates\":[{\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":3,\"candidatesTokenCount\":4,\"totalTokenCount\":7}}\n\n")
            .unwrap();
        let usage = result.usage.expect("usage should be extracted");

        assert!(result.completed);
        assert_eq!(usage.prompt_tokens, Some(3));
        assert_eq!(usage.completion_tokens, Some(4));
        assert_eq!(usage.total_tokens, Some(7));
    }
}
