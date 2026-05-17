use serde_json::Value;

use crate::llm_proxy::audit::TokenUsage;

use super::common::{child, finalize, nested_number_any, number_any};

pub(super) fn usage(value: Option<&Value>, semantic: &'static str) -> Option<TokenUsage> {
    let object = value?.as_object()?;
    let input_details = child(value?, &["prompt_tokens_details", "input_tokens_details", "input_token_details"]);
    let output_details = child(value?, &["completion_tokens_details", "output_tokens_details", "output_token_details"]);
    finalize(TokenUsage {
        prompt_tokens: number_any(value?, &["prompt_tokens", "input_tokens"]),
        completion_tokens: number_any(value?, &["completion_tokens", "output_tokens"]),
        total_tokens: number_any(value?, &["total_tokens"]),
        cache_creation_input_tokens: nested_number_any(input_details, &["cached_creation_tokens", "cache_creation_tokens"]),
        cache_read_input_tokens: nested_number_any(input_details, &["cached_tokens"]),
        input_text_tokens: nested_number_any(input_details, &["text_tokens"]),
        input_audio_tokens: nested_number_any(input_details, &["audio_tokens"]),
        input_image_tokens: nested_number_any(input_details, &["image_tokens"]),
        output_text_tokens: nested_number_any(output_details, &["text_tokens"]),
        output_audio_tokens: nested_number_any(output_details, &["audio_tokens"]),
        output_image_tokens: nested_number_any(output_details, &["image_tokens"]),
        reasoning_tokens: nested_number_any(output_details, &["reasoning_tokens"]),
        cache_creation_5m_input_tokens: None,
        cache_creation_1h_input_tokens: None,
        usage_source: Some("openai"),
        usage_semantic: Some(semantic),
    })
    .filter(|_| !object.is_empty())
}

pub(super) fn responses_usage(response: &Value) -> Option<TokenUsage> {
    usage(responses_usage_value(response), "responses")
}

fn responses_usage_value(response: &Value) -> Option<&Value> {
    response
        .get("usage")
        .or_else(|| response.get("response").and_then(|response| response.get("usage")))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{responses_usage, usage};

    #[test]
    fn extracts_openai_input_output_usage_fields() {
        let value = json!({
            "input_tokens": 10,
            "output_tokens": 4,
            "input_tokens_details": {"cached_tokens": 2, "text_tokens": 8, "audio_tokens": 1},
            "output_tokens_details": {"reasoning_tokens": 3, "text_tokens": 1}
        });

        let usage = usage(Some(&value), "audio").expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(10));
        assert_eq!(usage.completion_tokens, Some(4));
        assert_eq!(usage.total_tokens, Some(14));
        assert_eq!(usage.cache_read_input_tokens, Some(2));
        assert_eq!(usage.input_text_tokens, Some(8));
        assert_eq!(usage.input_audio_tokens, Some(1));
        assert_eq!(usage.output_text_tokens, Some(1));
        assert_eq!(usage.reasoning_tokens, Some(3));
        assert_eq!(usage.usage_semantic, Some("audio"));
    }

    #[test]
    fn extracts_openai_responses_usage_from_completed_stream_event() {
        let chunk = json!({
            "type": "response.completed",
            "response": {
                "usage": {
                    "input_tokens": 12,
                    "output_tokens": 7,
                    "total_tokens": 19,
                    "input_tokens_details": {"cached_tokens": 3}
                }
            }
        });

        let usage = responses_usage(&chunk).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(12));
        assert_eq!(usage.completion_tokens, Some(7));
        assert_eq!(usage.total_tokens, Some(19));
        assert_eq!(usage.cache_read_input_tokens, Some(3));
        assert_eq!(usage.usage_semantic, Some("responses"));
    }
}
