mod extractors;

use extractors::{OutputDelta, estimate_request_tokens, output_delta, supports_estimation, usage_semantic};
use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use crate::llm_proxy::{LlmProxyError, audit::TokenUsage};

use super::token_estimator::estimate_text_tokens;

#[cfg(test)]
mod tests;

pub(super) const ESTIMATED_USAGE_SOURCE: &str = "estimated_from_stream_delta";
pub(super) const ESTIMATED_REQUEST_USAGE_SOURCE: &str = "estimated_from_request_body";

pub(super) struct StreamUsageEstimator {
    format: ApiFormat,
    prompt_tokens: i64,
    model: String,
    buffer: Vec<u8>,
    output: String,
    output_image_tokens: i64,
    gemini_previous_output: String,
}

impl StreamUsageEstimator {
    pub(super) fn new(format: ApiFormat, request: &Value, model: &str) -> Self {
        Self {
            format,
            prompt_tokens: estimate_request_tokens(format, request, model),
            model: model.to_owned(),
            buffer: Vec::new(),
            output: String::new(),
            output_image_tokens: 0,
            gemini_previous_output: String::new(),
        }
    }

    pub(super) fn consume(&mut self, bytes: &[u8]) -> Result<(), LlmProxyError> {
        if !supports_estimation(self.format) {
            return Ok(());
        }
        self.buffer.extend_from_slice(bytes);
        while let Some(line) = self.next_line() {
            self.consume_line(&line)?;
        }
        Ok(())
    }

    pub(super) fn consume_chunk(&mut self, chunk: &Value) {
        if !supports_estimation(self.format) {
            return;
        }
        if let Some(delta) = output_delta(self.format, chunk, &mut self.gemini_previous_output) {
            self.apply_output_delta(delta);
        }
    }

    pub(super) fn estimated_usage(&self) -> Option<TokenUsage> {
        let output_text_tokens = estimate_text_tokens(&self.model, &self.output);
        let completion_tokens = output_text_tokens + self.output_image_tokens;
        if completion_tokens == 0 {
            return None;
        }
        Some(TokenUsage {
            prompt_tokens: Some(self.prompt_tokens),
            completion_tokens: Some(completion_tokens),
            total_tokens: Some(self.prompt_tokens + completion_tokens),
            output_text_tokens: positive(output_text_tokens),
            output_image_tokens: positive(self.output_image_tokens),
            usage_source: Some(ESTIMATED_USAGE_SOURCE),
            usage_semantic: Some(usage_semantic(self.format)),
            ..TokenUsage::default()
        })
    }

    pub(super) fn apply_to_usage(&self, current: Option<TokenUsage>, protocol_completed: bool) -> Option<TokenUsage> {
        let Some(estimated) = self.estimated_usage() else {
            return current.or_else(|| self.estimated_request_usage());
        };
        Some(match current {
            Some(current) => merge_estimated_usage(current, estimated, self.format, protocol_completed),
            None => estimated,
        })
    }

    pub(super) fn finish(&mut self) -> Result<(), LlmProxyError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let line = std::mem::take(&mut self.buffer);
        self.consume_line(&line)
    }

    fn estimated_request_usage(&self) -> Option<TokenUsage> {
        if self.prompt_tokens <= 0 {
            return None;
        }
        Some(TokenUsage {
            prompt_tokens: Some(self.prompt_tokens),
            completion_tokens: Some(0),
            total_tokens: Some(self.prompt_tokens),
            usage_source: Some(ESTIMATED_REQUEST_USAGE_SOURCE),
            usage_semantic: Some(usage_semantic(self.format)),
            ..TokenUsage::default()
        })
    }

    fn apply_output_delta(&mut self, delta: OutputDelta) {
        self.output.push_str(&delta.text);
        self.output_image_tokens += delta.image_tokens;
    }

    fn next_line(&mut self) -> Option<Vec<u8>> {
        let position = self.buffer.iter().position(|byte| *byte == b'\n')?;
        Some(self.buffer.drain(..=position).collect())
    }

    fn consume_line(&mut self, line: &[u8]) -> Result<(), LlmProxyError> {
        let line = std::str::from_utf8(line).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        let Some(payload) = line.trim_end_matches(['\r', '\n']).strip_prefix("data:") else {
            return Ok(());
        };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            return Ok(());
        }
        let chunk = serde_json::from_str::<Value>(payload).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
        self.consume_chunk(&chunk);
        Ok(())
    }
}

fn merge_estimated_usage(mut current: TokenUsage, estimated: TokenUsage, format: ApiFormat, protocol_completed: bool) -> TokenUsage {
    let mut used_estimate = false;
    if should_replace_completion(current, estimated, format, protocol_completed) {
        current.completion_tokens = estimated.completion_tokens;
        current.output_text_tokens = estimated.output_text_tokens;
        current.output_image_tokens = estimated.output_image_tokens;
        used_estimate = true;
    }
    if missing_or_zero(current.prompt_tokens) && !missing_or_zero(current.completion_tokens) {
        current.prompt_tokens = estimated.prompt_tokens;
        used_estimate = true;
    }
    current.total_tokens = merged_total_tokens(current, used_estimate);
    if used_estimate {
        current.usage_source = estimated.usage_source;
        current.usage_semantic = current.usage_semantic.or(estimated.usage_semantic);
    }
    current
}

fn should_replace_completion(current: TokenUsage, estimated: TokenUsage, format: ApiFormat, protocol_completed: bool) -> bool {
    if missing_or_zero(current.completion_tokens) {
        return true;
    }
    matches!(format, ApiFormat::ClaudeChat) && !protocol_completed && estimated.completion_tokens > current.completion_tokens
}

fn merged_total_tokens(usage: TokenUsage, used_estimate: bool) -> Option<i64> {
    if !used_estimate && !missing_or_zero(usage.total_tokens) {
        return usage.total_tokens;
    }
    match (usage.prompt_tokens, usage.completion_tokens) {
        (Some(prompt), Some(completion)) => Some(prompt + completion),
        _ => usage.total_tokens,
    }
}

fn missing_or_zero(value: Option<i64>) -> bool {
    value.unwrap_or_default() <= 0
}

fn positive(value: i64) -> Option<i64> {
    (value > 0).then_some(value)
}
