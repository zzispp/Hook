use serde_json::Value;

use crate::llm_proxy::audit::TokenUsage;

use super::common::{child, finalize, number, sum_optional};

pub(super) fn usage(response: &Value) -> Option<TokenUsage> {
    let metadata = response
        .get("usageMetadata")
        .or_else(|| response.get("usage_metadata"))
        .or_else(|| response.get("candidates")?.get(0)?.get("usageMetadata"))?;
    let object = metadata.as_object()?;
    let prompt_tokens = sum_optional(number(object.get("promptTokenCount")), number(object.get("toolUsePromptTokenCount")));
    let completion_tokens = sum_optional(number(object.get("candidatesTokenCount")), number(object.get("thoughtsTokenCount")));
    finalize(TokenUsage {
        prompt_tokens,
        completion_tokens,
        total_tokens: number(object.get("totalTokenCount")),
        cache_creation_input_tokens: None,
        cache_read_input_tokens: number(object.get("cachedContentTokenCount")),
        input_text_tokens: input_modality_tokens(metadata, "TEXT"),
        input_audio_tokens: input_modality_tokens(metadata, "AUDIO"),
        input_image_tokens: input_modality_tokens(metadata, "IMAGE"),
        output_text_tokens: output_modality_tokens(metadata, "TEXT"),
        output_audio_tokens: output_modality_tokens(metadata, "AUDIO"),
        output_image_tokens: output_modality_tokens(metadata, "IMAGE"),
        reasoning_tokens: number(object.get("thoughtsTokenCount")),
        cache_creation_5m_input_tokens: None,
        cache_creation_1h_input_tokens: None,
        usage_source: Some("google"),
        usage_semantic: Some("gemini"),
    })
}

fn input_modality_tokens(metadata: &Value, modality: &str) -> Option<i64> {
    sum_optional(
        modality_tokens(child(metadata, &["promptTokensDetails", "prompt_tokens_details"]), modality),
        modality_tokens(child(metadata, &["toolUsePromptTokensDetails", "tool_use_prompt_tokens_details"]), modality),
    )
}

fn output_modality_tokens(metadata: &Value, modality: &str) -> Option<i64> {
    modality_tokens(child(metadata, &["candidatesTokensDetails", "candidates_tokens_details"]), modality)
}

fn modality_tokens(value: Option<&Value>, modality: &str) -> Option<i64> {
    let details = value?.as_array()?;
    let total: i64 = details
        .iter()
        .filter(|detail| modality_matches(detail, modality))
        .filter_map(|detail| number(detail.get("tokenCount").or_else(|| detail.get("token_count"))))
        .sum();
    (total > 0).then_some(total)
}

fn modality_matches(detail: &Value, modality: &str) -> bool {
    detail.get("modality").and_then(Value::as_str) == Some(modality)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::usage;

    #[test]
    fn extracts_gemini_thinking_tool_and_modality_usage() {
        let response = json!({
            "usageMetadata": {
                "promptTokenCount": 10,
                "toolUsePromptTokenCount": 3,
                "candidatesTokenCount": 6,
                "thoughtsTokenCount": 4,
                "totalTokenCount": 23,
                "cachedContentTokenCount": 2,
                "promptTokensDetails": [
                    {"modality": "TEXT", "tokenCount": 8},
                    {"modality": "AUDIO", "tokenCount": 2}
                ],
                "toolUsePromptTokensDetails": [
                    {"modality": "TEXT", "tokenCount": 3}
                ],
                "candidatesTokensDetails": [
                    {"modality": "TEXT", "tokenCount": 5},
                    {"modality": "IMAGE", "tokenCount": 1}
                ]
            }
        });

        let usage = usage(&response).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(13));
        assert_eq!(usage.completion_tokens, Some(10));
        assert_eq!(usage.total_tokens, Some(23));
        assert_eq!(usage.cache_read_input_tokens, Some(2));
        assert_eq!(usage.input_text_tokens, Some(11));
        assert_eq!(usage.input_audio_tokens, Some(2));
        assert_eq!(usage.output_text_tokens, Some(5));
        assert_eq!(usage.output_image_tokens, Some(1));
        assert_eq!(usage.reasoning_tokens, Some(4));
    }
}
