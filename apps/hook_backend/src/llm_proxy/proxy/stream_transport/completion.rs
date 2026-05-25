use proxy::format_conversion::ApiFormat;
use serde_json::{Value, json};

use crate::llm_proxy::audit::TokenUsage;

use super::event::render_stream_event;

pub(super) fn response_id_from_chunk(chunk: &Value, format: ApiFormat) -> Option<String> {
    if format != ApiFormat::OpenAiResponses {
        return None;
    }
    chunk
        .get("response")
        .and_then(|response| response.get("id"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

pub(super) fn synthetic_openai_responses_completion(response_id: Option<&str>, usage: Option<TokenUsage>) -> req::Bytes {
    render_stream_event(&synthetic_openai_responses_completion_event(response_id, usage), ApiFormat::OpenAiResponses)
}

pub(super) fn synthetic_openai_responses_completion_event(response_id: Option<&str>, usage: Option<TokenUsage>) -> Value {
    json!({
        "type": "response.completed",
        "response": {
            "id": response_id.unwrap_or_default(),
            "usage": usage.and_then(usage_json),
        }
    })
}

fn usage_json(usage: TokenUsage) -> Option<Value> {
    Some(json!({
        "input_tokens": usage.prompt_tokens?,
        "output_tokens": usage.completion_tokens?,
        "total_tokens": usage.total_tokens.unwrap_or_else(|| {
            usage.prompt_tokens.unwrap_or_default().saturating_add(usage.completion_tokens.unwrap_or_default())
        }),
        "input_tokens_details": input_tokens_details(usage),
        "output_tokens_details": output_tokens_details(usage),
    }))
}

fn input_tokens_details(usage: TokenUsage) -> Value {
    json!({
        "cached_tokens": usage.cache_read_input_tokens.unwrap_or_default(),
    })
}

fn output_tokens_details(usage: TokenUsage) -> Value {
    json!({
        "reasoning_tokens": usage.reasoning_tokens.unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::llm_proxy::{audit::TokenUsage, proxy::stream_transport::completion::synthetic_openai_responses_completion};

    #[test]
    fn synthetic_openai_responses_completion_uses_real_response_id_and_usage() {
        let bytes = synthetic_openai_responses_completion(
            Some("resp_123"),
            Some(TokenUsage {
                prompt_tokens: Some(7),
                completion_tokens: Some(5),
                total_tokens: Some(12),
                cache_read_input_tokens: Some(3),
                reasoning_tokens: Some(2),
                ..TokenUsage::default()
            }),
        );

        let payload = payload_json(&bytes);

        assert_eq!(payload["type"], "response.completed");
        assert_eq!(payload["response"]["id"], "resp_123");
        assert_eq!(payload["response"]["usage"]["input_tokens"], 7);
        assert_eq!(payload["response"]["usage"]["output_tokens"], 5);
        assert_eq!(payload["response"]["usage"]["total_tokens"], 12);
        assert_eq!(payload["response"]["usage"]["input_tokens_details"]["cached_tokens"], 3);
        assert_eq!(payload["response"]["usage"]["output_tokens_details"]["reasoning_tokens"], 2);
    }

    #[test]
    fn synthetic_openai_responses_completion_uses_empty_id_without_upstream_id() {
        let bytes = synthetic_openai_responses_completion(None, None);
        let payload = payload_json(&bytes);

        assert_eq!(payload["response"]["id"], "");
        assert!(payload["response"]["usage"].is_null());
    }

    fn payload_json(bytes: &[u8]) -> Value {
        let text = std::str::from_utf8(bytes).unwrap();
        let payload = text.lines().find_map(|line| line.strip_prefix("data: ")).expect("SSE data line");
        serde_json::from_str(payload).unwrap()
    }
}
