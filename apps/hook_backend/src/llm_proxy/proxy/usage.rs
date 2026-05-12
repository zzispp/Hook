use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use crate::llm_proxy::audit::TokenUsage;

pub fn from_response_bytes(bytes: &[u8], format: ApiFormat) -> Option<TokenUsage> {
    let response: Value = serde_json::from_slice(bytes).ok()?;
    from_response(&response, format)
}

pub fn from_stream_chunk(chunk: &Value, format: ApiFormat) -> Option<TokenUsage> {
    from_response(chunk, format)
}

pub fn merge(current: Option<TokenUsage>, incoming: TokenUsage) -> Option<TokenUsage> {
    let current = current.unwrap_or_default();
    usage(TokenUsage {
        prompt_tokens: incoming.prompt_tokens.or(current.prompt_tokens),
        completion_tokens: incoming.completion_tokens.or(current.completion_tokens),
        total_tokens: incoming.total_tokens.or(current.total_tokens),
        cache_creation_input_tokens: incoming.cache_creation_input_tokens.or(current.cache_creation_input_tokens),
        cache_read_input_tokens: incoming.cache_read_input_tokens.or(current.cache_read_input_tokens),
    })
}

fn from_response(response: &Value, format: ApiFormat) -> Option<TokenUsage> {
    match format {
        ApiFormat::OpenAiChat => openai_usage(response.get("usage")),
        ApiFormat::OpenAiResponses => openai_responses_usage(response),
        ApiFormat::ClaudeChat => claude_usage(response),
        ApiFormat::GeminiChat => gemini_usage(response),
    }
}

fn openai_usage(value: Option<&Value>) -> Option<TokenUsage> {
    let object = value?.as_object()?;
    usage(TokenUsage {
        prompt_tokens: number(object.get("prompt_tokens")),
        completion_tokens: number(object.get("completion_tokens")),
        total_tokens: number(object.get("total_tokens")),
        cache_creation_input_tokens: None,
        cache_read_input_tokens: nested_number(object.get("prompt_tokens_details"), "cached_tokens"),
    })
}

fn openai_responses_usage(response: &Value) -> Option<TokenUsage> {
    let object = response.get("usage")?.as_object()?;
    usage(TokenUsage {
        prompt_tokens: number(object.get("input_tokens")),
        completion_tokens: number(object.get("output_tokens")),
        total_tokens: number(object.get("total_tokens")),
        cache_creation_input_tokens: None,
        cache_read_input_tokens: nested_number(object.get("input_tokens_details"), "cached_tokens"),
    })
}

fn claude_usage(response: &Value) -> Option<TokenUsage> {
    let usage_value = response
        .get("usage")
        .or_else(|| response.get("message").and_then(|message| message.get("usage")))?;
    let object = usage_value.as_object()?;
    usage(TokenUsage {
        prompt_tokens: number(object.get("input_tokens")),
        completion_tokens: number(object.get("output_tokens")),
        total_tokens: None,
        cache_creation_input_tokens: claude_cache_creation(usage_value),
        cache_read_input_tokens: number(object.get("cache_read_input_tokens")),
    })
}

fn gemini_usage(response: &Value) -> Option<TokenUsage> {
    let metadata = response
        .get("usageMetadata")
        .or_else(|| response.get("usage_metadata"))
        .or_else(|| response.get("candidates")?.get(0)?.get("usageMetadata"))?;
    let object = metadata.as_object()?;
    usage(TokenUsage {
        prompt_tokens: number(object.get("promptTokenCount")),
        completion_tokens: number(object.get("candidatesTokenCount")),
        total_tokens: number(object.get("totalTokenCount")),
        cache_creation_input_tokens: None,
        cache_read_input_tokens: number(object.get("cachedContentTokenCount")),
    })
}

fn usage(mut usage: TokenUsage) -> Option<TokenUsage> {
    usage.total_tokens = usage.total_tokens.or_else(|| Some(usage.prompt_tokens? + usage.completion_tokens?));
    usage.has_any_value().then_some(usage)
}

fn claude_cache_creation(value: &Value) -> Option<i64> {
    number(value.get("cache_creation_input_tokens")).or_else(|| {
        let creation = value.get("cache_creation")?;
        Some(number(creation.get("ephemeral_5m_input_tokens")).unwrap_or(0) + number(creation.get("ephemeral_1h_input_tokens")).unwrap_or(0))
    })
}

fn nested_number(value: Option<&Value>, key: &str) -> Option<i64> {
    number(value?.get(key))
}

fn number(value: Option<&Value>) -> Option<i64> {
    value?.as_i64().or_else(|| value?.as_u64().and_then(|number| i64::try_from(number).ok()))
}

impl TokenUsage {
    fn has_any_value(&self) -> bool {
        self.prompt_tokens.is_some()
            || self.completion_tokens.is_some()
            || self.total_tokens.is_some()
            || self.cache_creation_input_tokens.is_some()
            || self.cache_read_input_tokens.is_some()
    }
}
