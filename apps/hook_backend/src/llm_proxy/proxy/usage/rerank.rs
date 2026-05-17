use serde_json::Value;

use crate::llm_proxy::audit::TokenUsage;

use super::common::{child, finalize, number_any};

pub(super) fn usage(response: &Value) -> Option<TokenUsage> {
    usage_object(response.get("usage")).or_else(|| meta_tokens(response))
}

fn usage_object(value: Option<&Value>) -> Option<TokenUsage> {
    let usage = value?;
    let prompt = number_any(usage, &["prompt_tokens", "input_tokens"]);
    let completion = number_any(usage, &["completion_tokens", "output_tokens"]);
    let total = number_any(usage, &["total_tokens", "totalTokens"]);
    let prompt = prompt.or_else(|| total.filter(|_| completion.is_none()));
    let completion = completion.or_else(|| prompt.and(Some(0)));
    finalize(TokenUsage {
        prompt_tokens: prompt,
        completion_tokens: completion,
        total_tokens: total,
        usage_source: Some("rerank"),
        usage_semantic: Some("rerank"),
        ..TokenUsage::default()
    })
}

fn meta_tokens(response: &Value) -> Option<TokenUsage> {
    let tokens = child(response.get("meta")?, &["tokens"])?;
    let prompt = number_any(tokens, &["input_tokens", "inputTokens"]);
    let completion = number_any(tokens, &["output_tokens", "outputTokens"]);
    finalize(TokenUsage {
        prompt_tokens: prompt,
        completion_tokens: completion,
        total_tokens: number_any(tokens, &["total_tokens", "totalTokens"]),
        usage_source: Some("rerank"),
        usage_semantic: Some("rerank"),
        ..TokenUsage::default()
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::usage;

    #[test]
    fn extracts_rerank_usage_total_as_prompt_tokens() {
        let response = json!({"usage": {"total_tokens": 17}});

        let usage = usage(&response).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(17));
        assert_eq!(usage.completion_tokens, Some(0));
        assert_eq!(usage.total_tokens, Some(17));
        assert_eq!(usage.usage_semantic, Some("rerank"));
    }

    #[test]
    fn extracts_rerank_meta_tokens() {
        let response = json!({"meta": {"tokens": {"input_tokens": 9, "output_tokens": 2}}});

        let usage = usage(&response).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(9));
        assert_eq!(usage.completion_tokens, Some(2));
        assert_eq!(usage.total_tokens, Some(11));
    }
}
