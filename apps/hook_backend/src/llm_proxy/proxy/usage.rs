mod claude;
mod common;
mod gemini;
mod openai;
mod rerank;

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
    common::finalize(TokenUsage {
        prompt_tokens: incoming.prompt_tokens.or(current.prompt_tokens),
        completion_tokens: incoming.completion_tokens.or(current.completion_tokens),
        total_tokens: incoming.total_tokens.or(current.total_tokens),
        cache_creation_input_tokens: incoming.cache_creation_input_tokens.or(current.cache_creation_input_tokens),
        cache_read_input_tokens: incoming.cache_read_input_tokens.or(current.cache_read_input_tokens),
        input_text_tokens: incoming.input_text_tokens.or(current.input_text_tokens),
        input_audio_tokens: incoming.input_audio_tokens.or(current.input_audio_tokens),
        input_image_tokens: incoming.input_image_tokens.or(current.input_image_tokens),
        output_text_tokens: incoming.output_text_tokens.or(current.output_text_tokens),
        output_audio_tokens: incoming.output_audio_tokens.or(current.output_audio_tokens),
        output_image_tokens: incoming.output_image_tokens.or(current.output_image_tokens),
        reasoning_tokens: incoming.reasoning_tokens.or(current.reasoning_tokens),
        cache_creation_5m_input_tokens: incoming.cache_creation_5m_input_tokens.or(current.cache_creation_5m_input_tokens),
        cache_creation_1h_input_tokens: incoming.cache_creation_1h_input_tokens.or(current.cache_creation_1h_input_tokens),
        usage_source: incoming.usage_source.or(current.usage_source),
        usage_semantic: incoming.usage_semantic.or(current.usage_semantic),
    })
}

fn from_response(response: &Value, format: ApiFormat) -> Option<TokenUsage> {
    match format {
        ApiFormat::OpenAiChat => openai::usage(response.get("usage"), "openai"),
        ApiFormat::OpenAiCompletion => openai::usage(response.get("usage"), "completion"),
        ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact => openai::responses_usage(response),
        ApiFormat::OpenAiEmbedding => openai::usage(response.get("usage"), "embedding"),
        ApiFormat::OpenAiImage => openai::usage(response.get("usage"), "image"),
        ApiFormat::OpenAiAudio => openai::usage(response.get("usage"), "audio"),
        ApiFormat::OpenAiModeration => openai::usage(response.get("usage"), "moderation"),
        ApiFormat::ClaudeChat => claude::usage(response),
        ApiFormat::GeminiChat | ApiFormat::GeminiEmbedding => gemini::usage(response),
        ApiFormat::Rerank => rerank::usage(response),
        ApiFormat::OpenAiRealtime | ApiFormat::GeminiVideo => None,
    }
}

#[cfg(test)]
mod tests {
    use proxy::format_conversion::ApiFormat;
    use serde_json::json;

    use super::from_response_bytes;

    #[test]
    fn extracts_openai_completion_usage() {
        let bytes = serde_json::to_vec(&json!({
            "usage": {"prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8}
        }))
        .unwrap();

        let usage = from_response_bytes(&bytes, ApiFormat::OpenAiCompletion).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(5));
        assert_eq!(usage.completion_tokens, Some(3));
        assert_eq!(usage.total_tokens, Some(8));
        assert_eq!(usage.usage_semantic, Some("completion"));
    }

    #[test]
    fn extracts_gemini_embedding_usage_metadata_when_provider_returns_it() {
        let bytes = serde_json::to_vec(&json!({
            "embedding": {"values": [0.1, 0.2]},
            "usageMetadata": {"promptTokenCount": 6, "totalTokenCount": 6}
        }))
        .unwrap();

        let usage = from_response_bytes(&bytes, ApiFormat::GeminiEmbedding).expect("usage should be extracted");

        assert_eq!(usage.prompt_tokens, Some(6));
        assert_eq!(usage.total_tokens, Some(6));
        assert_eq!(usage.usage_source, Some("google"));
    }

    #[test]
    fn leaves_missing_usage_visible_when_provider_returns_no_usage() {
        let bytes = serde_json::to_vec(&json!({"data": [{"embedding": [0.1]}]})).unwrap();

        assert!(from_response_bytes(&bytes, ApiFormat::OpenAiEmbedding).is_none());
    }
}
