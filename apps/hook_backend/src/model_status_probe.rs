use async_trait::async_trait;
use axum::http::HeaderMap;
use model_status::application::{
    ModelStatusProbe, ModelStatusProbeInput, ModelStatusProbeOptions, ModelStatusProbeOutput, ModelStatusProbeResult, ModelStatusRunStatus,
};
use serde_json::{Value, json};
use std::time::Instant;

use crate::llm_proxy::{
    CLAUDE_CHAT_FORMAT, CurrentApiToken, GEMINI_CHAT_FORMAT, LlmProxyState, OPENAI_CHAT_FORMAT, OPENAI_CLI_FORMAT, ProviderKeyProbeSlotOptions,
    ProxyJsonRequest, proxy_json,
};

pub(crate) const DEGRADED_AFTER_MS: i64 = 6_000;

#[derive(Clone)]
pub(crate) struct LlmProxyModelStatusProbe {
    state: LlmProxyState,
}

impl LlmProxyModelStatusProbe {
    pub(crate) const fn new(state: LlmProxyState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ModelStatusProbe for LlmProxyModelStatusProbe {
    async fn probe(&self, input: ModelStatusProbeInput, options: ModelStatusProbeOptions) -> ModelStatusProbeResult {
        let Some(api_format) = supported_api_format(&input.api_format) else {
            return completed_error(format!("unsupported model status api_format: {}", input.api_format));
        };
        let Some(body) = probe_body(&input) else {
            return completed_error(format!("unsupported model status api_format: {}", input.api_format));
        };
        let started = Instant::now();
        let request = ProxyJsonRequest::new(self.state.clone(), CurrentApiToken(input.token), HeaderMap::new(), body, api_format, true)
            .with_provider_key_probe_slot_options(ProviderKeyProbeSlotOptions {
                min_interval_seconds: options.provider_key_min_interval_seconds,
                wait_timeout_seconds: options.provider_key_probe_wait_timeout_seconds,
            });
        match proxy_json(request).await {
            Ok(response) => ModelStatusProbeResult::Completed(classify_response(response.status().as_u16(), elapsed_ms(started))),
            Err(error) => completed_error(error.to_string()),
        }
    }
}

fn probe_body(input: &ModelStatusProbeInput) -> Option<Value> {
    match input.api_format.as_str() {
        OPENAI_CHAT_FORMAT => Some(openai_chat_body(&input.model_name)),
        OPENAI_CLI_FORMAT => Some(openai_cli_body(&input.model_name)),
        CLAUDE_CHAT_FORMAT => Some(claude_chat_body(&input.model_name)),
        GEMINI_CHAT_FORMAT => Some(gemini_chat_body(&input.model_name)),
        _ => None,
    }
}

fn supported_api_format(value: &str) -> Option<&'static str> {
    match value {
        OPENAI_CHAT_FORMAT => Some(OPENAI_CHAT_FORMAT),
        OPENAI_CLI_FORMAT => Some(OPENAI_CLI_FORMAT),
        CLAUDE_CHAT_FORMAT => Some(CLAUDE_CHAT_FORMAT),
        GEMINI_CHAT_FORMAT => Some(GEMINI_CHAT_FORMAT),
        _ => None,
    }
}

fn openai_chat_body(model: &str) -> Value {
    json!({
        "model": model,
        "messages": [{ "role": "user", "content": "ping" }],
        "max_tokens": 8,
        "temperature": 0,
        "stream": false
    })
}

fn openai_cli_body(model: &str) -> Value {
    json!({
        "model": model,
        "input": "ping",
        "max_output_tokens": 8,
        "temperature": 0,
        "stream": false
    })
}

fn claude_chat_body(model: &str) -> Value {
    json!({
        "model": model,
        "messages": [{ "role": "user", "content": "ping" }],
        "max_tokens": 8,
        "temperature": 0,
        "stream": false
    })
}

fn gemini_chat_body(model: &str) -> Value {
    json!({
        "model": model,
        "contents": [{ "role": "user", "parts": [{ "text": "ping" }] }],
        "generationConfig": { "maxOutputTokens": 8, "temperature": 0 },
        "stream": false
    })
}

pub(crate) fn classify_response(status_code: u16, latency_ms: i64) -> ModelStatusProbeOutput {
    let status = if (200..300).contains(&status_code) {
        success_status(latency_ms)
    } else {
        ModelStatusRunStatus::Failed
    };
    ModelStatusProbeOutput {
        status,
        latency_ms: Some(latency_ms),
        status_code: Some(i32::from(status_code)),
        message: failed_message(status_code, status),
    }
}

fn success_status(latency_ms: i64) -> ModelStatusRunStatus {
    if latency_ms <= DEGRADED_AFTER_MS {
        ModelStatusRunStatus::Operational
    } else {
        ModelStatusRunStatus::Degraded
    }
}

fn failed_message(status_code: u16, status: ModelStatusRunStatus) -> Option<String> {
    (status == ModelStatusRunStatus::Failed).then(|| format!("HTTP status {status_code}"))
}

fn error_output(message: String) -> ModelStatusProbeOutput {
    ModelStatusProbeOutput {
        status: ModelStatusRunStatus::Error,
        latency_ms: None,
        status_code: None,
        message: Some(message),
    }
}

fn completed_error(message: String) -> ModelStatusProbeResult {
    ModelStatusProbeResult::Completed(error_output(message))
}

fn elapsed_ms(started: Instant) -> i64 {
    i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_fast_success_as_operational() {
        let output = classify_response(200, DEGRADED_AFTER_MS);
        assert_eq!(output.status, ModelStatusRunStatus::Operational);
        assert_eq!(output.status_code, Some(200));
    }

    #[test]
    fn classifies_slow_success_as_degraded() {
        let output = classify_response(200, DEGRADED_AFTER_MS + 1);
        assert_eq!(output.status, ModelStatusRunStatus::Degraded);
    }

    #[test]
    fn classifies_non_2xx_as_failed() {
        let output = classify_response(500, 12);
        assert_eq!(output.status, ModelStatusRunStatus::Failed);
        assert_eq!(output.message, Some("HTTP status 500".into()));
    }
}
