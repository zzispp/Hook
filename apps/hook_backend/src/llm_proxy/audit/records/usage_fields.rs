use storage::provider::{RequestCandidateRecordInput, RequestCandidateRecordPatch, RequestRecordRecordPatch};
use types::model::PatchField;

use crate::llm_proxy::audit::{AttemptRecordInput, TokenUsage};

pub(super) fn candidate_patch(input: &AttemptRecordInput<'_>, patch: &mut RequestCandidateRecordPatch) {
    let usage = display_usage(input);
    patch.prompt_tokens = usage.and_then(|usage| usage.prompt_tokens);
    patch.completion_tokens = input.usage.and_then(|usage| usage.completion_tokens);
    patch.total_tokens = usage.and_then(|usage| usage.total_tokens);
    patch.cache_creation_input_tokens = input.usage.and_then(|usage| usage.cache_creation_input_tokens);
    patch.cache_read_input_tokens = input.usage.and_then(|usage| usage.cache_read_input_tokens);
    patch.input_text_tokens = input.usage.and_then(|usage| usage.input_text_tokens);
    patch.input_audio_tokens = input.usage.and_then(|usage| usage.input_audio_tokens);
    patch.input_image_tokens = input.usage.and_then(|usage| usage.input_image_tokens);
    patch.output_text_tokens = input.usage.and_then(|usage| usage.output_text_tokens);
    patch.output_audio_tokens = input.usage.and_then(|usage| usage.output_audio_tokens);
    patch.output_image_tokens = input.usage.and_then(|usage| usage.output_image_tokens);
    patch.reasoning_tokens = input.usage.and_then(|usage| usage.reasoning_tokens);
    patch.cache_creation_5m_input_tokens = input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens);
    patch.cache_creation_1h_input_tokens = input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens);
    patch.usage_source = input.usage.and_then(|usage| usage.usage_source.map(str::to_owned));
    patch.usage_semantic = input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned));
}

pub(super) fn candidate_input(input: &AttemptRecordInput<'_>, record: &mut RequestCandidateRecordInput) {
    let usage = display_usage(input);
    record.prompt_tokens = usage.and_then(|usage| usage.prompt_tokens);
    record.completion_tokens = input.usage.and_then(|usage| usage.completion_tokens);
    record.total_tokens = usage.and_then(|usage| usage.total_tokens);
    record.cache_creation_input_tokens = input.usage.and_then(|usage| usage.cache_creation_input_tokens);
    record.cache_read_input_tokens = input.usage.and_then(|usage| usage.cache_read_input_tokens);
    record.input_text_tokens = input.usage.and_then(|usage| usage.input_text_tokens);
    record.input_audio_tokens = input.usage.and_then(|usage| usage.input_audio_tokens);
    record.input_image_tokens = input.usage.and_then(|usage| usage.input_image_tokens);
    record.output_text_tokens = input.usage.and_then(|usage| usage.output_text_tokens);
    record.output_audio_tokens = input.usage.and_then(|usage| usage.output_audio_tokens);
    record.output_image_tokens = input.usage.and_then(|usage| usage.output_image_tokens);
    record.reasoning_tokens = input.usage.and_then(|usage| usage.reasoning_tokens);
    record.cache_creation_5m_input_tokens = input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens);
    record.cache_creation_1h_input_tokens = input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens);
    record.usage_source = input.usage.and_then(|usage| usage.usage_source.map(str::to_owned));
    record.usage_semantic = input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned));
}

pub(super) fn request_patch(input: &AttemptRecordInput<'_>, patch: &mut RequestRecordRecordPatch) {
    let usage = display_usage(input);
    patch.prompt_tokens = option_patch(usage.and_then(|usage| usage.prompt_tokens));
    patch.completion_tokens = option_patch(input.usage.and_then(|usage| usage.completion_tokens));
    patch.total_tokens = option_patch(usage.and_then(|usage| usage.total_tokens));
    patch.cache_creation_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_creation_input_tokens));
    patch.cache_read_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_read_input_tokens));
    patch.input_text_tokens = option_patch(input.usage.and_then(|usage| usage.input_text_tokens));
    patch.input_audio_tokens = option_patch(input.usage.and_then(|usage| usage.input_audio_tokens));
    patch.input_image_tokens = option_patch(input.usage.and_then(|usage| usage.input_image_tokens));
    patch.output_text_tokens = option_patch(input.usage.and_then(|usage| usage.output_text_tokens));
    patch.output_audio_tokens = option_patch(input.usage.and_then(|usage| usage.output_audio_tokens));
    patch.output_image_tokens = option_patch(input.usage.and_then(|usage| usage.output_image_tokens));
    patch.reasoning_tokens = option_patch(input.usage.and_then(|usage| usage.reasoning_tokens));
    patch.cache_creation_5m_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_creation_5m_input_tokens));
    patch.cache_creation_1h_input_tokens = option_patch(input.usage.and_then(|usage| usage.cache_creation_1h_input_tokens));
    patch.usage_source = option_patch(input.usage.and_then(|usage| usage.usage_source.map(str::to_owned)));
    patch.usage_semantic = option_patch(input.usage.and_then(|usage| usage.usage_semantic.map(str::to_owned)));
}

fn display_usage(input: &AttemptRecordInput<'_>) -> Option<TokenUsage> {
    input.usage.map(|usage| display_token_usage(usage, &input.candidate.trace.provider_api_format))
}

fn display_token_usage(mut usage: TokenUsage, provider_api_format: &str) -> TokenUsage {
    if !prompt_tokens_include_cache_tokens(&usage, provider_api_format) {
        return usage;
    }
    let Some(prompt_tokens) = usage.prompt_tokens else {
        return usage;
    };
    let cache_tokens = positive_token(usage.cache_creation_input_tokens) + positive_token(usage.cache_read_input_tokens);
    usage.prompt_tokens = Some((prompt_tokens - cache_tokens).max(0));
    usage.total_tokens = display_total_tokens(usage, cache_tokens);
    usage
}

fn display_total_tokens(usage: TokenUsage, cache_tokens: i64) -> Option<i64> {
    usage
        .total_tokens
        .map(|total_tokens| (total_tokens - cache_tokens).max(0))
        .or_else(|| Some(usage.prompt_tokens? + usage.completion_tokens?))
}

fn prompt_tokens_include_cache_tokens(usage: &TokenUsage, provider_api_format: &str) -> bool {
    if let Some(semantic) = usage.usage_semantic {
        if semantic == "anthropic" {
            return false;
        }
        if matches!(
            semantic,
            "openai" | "responses" | "completion" | "embedding" | "image" | "audio" | "moderation" | "realtime" | "gemini"
        ) {
            return true;
        }
    }
    api_format_includes_cache_tokens(provider_api_format)
}

fn api_format_includes_cache_tokens(api_format: &str) -> bool {
    let api_format = api_format.trim().to_ascii_lowercase();
    api_format == "openai"
        || api_format == "gemini"
        || api_format.starts_with("openai_")
        || api_format.starts_with("openai:")
        || api_format.starts_with("gemini_")
        || api_format.starts_with("gemini:")
}

fn positive_token(value: Option<i64>) -> i64 {
    value.unwrap_or(0).max(0)
}

fn option_patch<T>(value: Option<T>) -> PatchField<T> {
    match value {
        Some(value) => PatchField::Value(value),
        None => PatchField::Null,
    }
}

#[cfg(test)]
mod tests {
    use crate::llm_proxy::audit::TokenUsage;

    use super::display_token_usage;

    #[test]
    fn openai_display_usage_removes_cache_tokens() {
        let usage = display_token_usage(
            TokenUsage {
                prompt_tokens: Some(14_800),
                completion_tokens: Some(9),
                total_tokens: Some(14_809),
                cache_read_input_tokens: Some(14_329),
                usage_semantic: Some("responses"),
                ..Default::default()
            },
            "openai_cli",
        );

        assert_eq!(usage.prompt_tokens, Some(471));
        assert_eq!(usage.completion_tokens, Some(9));
        assert_eq!(usage.total_tokens, Some(480));
        assert_eq!(usage.cache_read_input_tokens, Some(14_329));
    }

    #[test]
    fn anthropic_display_usage_keeps_prompt_tokens() {
        let usage = display_token_usage(
            TokenUsage {
                prompt_tokens: Some(471),
                completion_tokens: Some(9),
                total_tokens: Some(480),
                cache_read_input_tokens: Some(14_329),
                usage_semantic: Some("anthropic"),
                ..Default::default()
            },
            "claude_chat",
        );

        assert_eq!(usage.prompt_tokens, Some(471));
        assert_eq!(usage.total_tokens, Some(480));
    }

    #[test]
    fn gemini_display_usage_removes_cache_tokens() {
        let usage = display_token_usage(
            TokenUsage {
                prompt_tokens: Some(100),
                completion_tokens: Some(10),
                total_tokens: Some(110),
                cache_read_input_tokens: Some(20),
                usage_semantic: Some("gemini"),
                ..Default::default()
            },
            "gemini_chat",
        );

        assert_eq!(usage.prompt_tokens, Some(80));
        assert_eq!(usage.completion_tokens, Some(10));
        assert_eq!(usage.total_tokens, Some(90));
        assert_eq!(usage.cache_read_input_tokens, Some(20));
    }

    #[test]
    fn input_only_display_usage_keeps_adjusted_total_tokens() {
        let usage = display_token_usage(
            TokenUsage {
                prompt_tokens: Some(6),
                total_tokens: Some(6),
                cache_read_input_tokens: Some(2),
                usage_semantic: Some("embedding"),
                ..Default::default()
            },
            "openai_embedding",
        );

        assert_eq!(usage.prompt_tokens, Some(4));
        assert_eq!(usage.completion_tokens, None);
        assert_eq!(usage.total_tokens, Some(4));
    }

    #[test]
    fn api_format_fallback_removes_cache_tokens_when_semantic_is_missing() {
        let usage = display_token_usage(
            TokenUsage {
                prompt_tokens: Some(30),
                completion_tokens: Some(5),
                total_tokens: Some(35),
                cache_creation_input_tokens: Some(10),
                ..Default::default()
            },
            "openai_audio_speech",
        );

        assert_eq!(usage.prompt_tokens, Some(20));
        assert_eq!(usage.total_tokens, Some(25));
    }
}
