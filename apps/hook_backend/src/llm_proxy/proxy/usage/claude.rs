use serde_json::Value;

use crate::llm_proxy::audit::TokenUsage;

use super::common::{finalize, number};

pub(super) fn usage(response: &Value) -> Option<TokenUsage> {
    let usage_value = response
        .get("usage")
        .or_else(|| response.get("message").and_then(|message| message.get("usage")))?;
    let object = usage_value.as_object()?;
    let cache_creation_5m = cache_creation_5m(usage_value);
    let cache_creation_1h = cache_creation_1h(usage_value);
    finalize(TokenUsage {
        prompt_tokens: number(object.get("input_tokens")),
        completion_tokens: number(object.get("output_tokens")),
        total_tokens: None,
        cache_creation_input_tokens: cache_creation(usage_value, cache_creation_5m, cache_creation_1h),
        cache_read_input_tokens: number(object.get("cache_read_input_tokens")),
        input_text_tokens: None,
        input_audio_tokens: None,
        input_image_tokens: None,
        output_text_tokens: None,
        output_audio_tokens: None,
        output_image_tokens: None,
        reasoning_tokens: None,
        cache_creation_5m_input_tokens: cache_creation_5m,
        cache_creation_1h_input_tokens: cache_creation_1h,
        usage_source: Some("anthropic"),
        usage_semantic: Some("anthropic"),
    })
}

fn cache_creation(value: &Value, cache_creation_5m: Option<i64>, cache_creation_1h: Option<i64>) -> Option<i64> {
    number(value.get("cache_creation_input_tokens")).or_else(|| super::common::sum_optional(cache_creation_5m, cache_creation_1h))
}

fn cache_creation_5m(value: &Value) -> Option<i64> {
    number(value.get("cache_creation").and_then(|creation| creation.get("ephemeral_5m_input_tokens")))
}

fn cache_creation_1h(value: &Value) -> Option<i64> {
    number(value.get("cache_creation").and_then(|creation| creation.get("ephemeral_1h_input_tokens")))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::usage;

    #[test]
    fn extracts_claude_cache_creation_split() {
        let response = json!({
            "usage": {
                "input_tokens": 11,
                "output_tokens": 6,
                "cache_creation": {
                    "ephemeral_5m_input_tokens": 3,
                    "ephemeral_1h_input_tokens": 4
                },
                "cache_read_input_tokens": 5
            }
        });

        let usage = usage(&response).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(11));
        assert_eq!(usage.completion_tokens, Some(6));
        assert_eq!(usage.total_tokens, Some(17));
        assert_eq!(usage.cache_creation_input_tokens, Some(7));
        assert_eq!(usage.cache_read_input_tokens, Some(5));
        assert_eq!(usage.cache_creation_5m_input_tokens, Some(3));
        assert_eq!(usage.cache_creation_1h_input_tokens, Some(4));
    }
}
