use proxy::format_conversion::ApiFormat;
use serde_json::json;

use crate::llm_proxy::audit::TokenUsage;

use super::StreamUsageEstimator;

#[test]
fn estimates_openai_responses_text_delta_usage() {
    let request = json!({"model":"gpt-5.5","input":[{"role":"user","content":"hello"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::OpenAiResponses, &request, "gpt-5.5");
    estimator
        .consume(b"data: {\"type\":\"response.output_text.delta\",\"delta\":\"world\"}\n\n")
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");

    assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
    assert_eq!(usage.usage_semantic, Some("responses"));
    assert_eq!(usage.total_tokens, Some(usage.prompt_tokens.unwrap() + usage.completion_tokens.unwrap()));
}

#[test]
fn estimates_openai_chat_tool_call_usage() {
    let request = json!({"model":"gpt-5.5","messages":[{"role":"user","content":"call tool"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::OpenAiChat, &request, "gpt-5.5");
    estimator
        .consume(
            b"data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"function\":{\"name\":\"run\",\"arguments\":\"{\\\"cmd\\\":\\\"ls\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
        )
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");

    assert_eq!(usage.usage_semantic, Some("openai"));
    assert!(usage.completion_tokens.unwrap() > 0);
}

#[test]
fn estimates_openai_chat_text_usage_before_done_marker() {
    let request = json!({"model":"gpt-5.5","messages":[{"role":"user","content":"hello"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::OpenAiChat, &request, "gpt-5.5");
    estimator
        .consume(b"data: {\"choices\":[{\"delta\":{\"content\":\"world\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n\n")
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");

    assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
    assert_eq!(usage.usage_semantic, Some("openai"));
}

#[test]
fn estimates_claude_text_delta_usage() {
    let request = json!({"model":"claude-sonnet-4","messages":[{"role":"user","content":"hello"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::ClaudeChat, &request, "claude-sonnet-4");
    estimator
        .consume(b"data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"world\"}}\n\n")
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");

    assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
    assert_eq!(usage.usage_semantic, Some("anthropic"));
    assert!(usage.prompt_tokens.unwrap() > 0);
    assert!(usage.completion_tokens.unwrap() > 0);
}

#[test]
fn estimates_claude_tool_and_thinking_delta_usage() {
    let request = json!({"model":"claude-sonnet-4","messages":[{"role":"user","content":"use a tool"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::ClaudeChat, &request, "claude-sonnet-4");
    estimator
        .consume(b"data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"plan\"}}\n\n")
        .unwrap();
    estimator
        .consume(b"data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"city\\\":\\\"SF\\\"}\"}}\n\n")
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");

    assert_eq!(usage.usage_semantic, Some("anthropic"));
    assert!(usage.completion_tokens.unwrap() > 0);
}

#[test]
fn incomplete_claude_stream_can_replace_lower_reported_completion_tokens() {
    let request = json!({"model":"claude-sonnet-4","messages":[{"role":"user","content":"hello"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::ClaudeChat, &request, "claude-sonnet-4");
    estimator
        .consume(b"data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"one two three four five six seven eight\"}}\n\n")
        .unwrap();

    let usage = estimator
        .apply_to_usage(
            Some(TokenUsage {
                prompt_tokens: Some(12),
                completion_tokens: Some(1),
                cache_read_input_tokens: Some(3),
                usage_source: Some("anthropic"),
                usage_semantic: Some("anthropic"),
                ..TokenUsage::default()
            }),
            false,
        )
        .expect("usage should stay present");

    assert_eq!(usage.prompt_tokens, Some(12));
    assert_eq!(usage.cache_read_input_tokens, Some(3));
    assert!(usage.completion_tokens.unwrap() > 1);
    assert_eq!(usage.total_tokens, Some(12 + usage.completion_tokens.unwrap()));
    assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
    assert_eq!(usage.usage_semantic, Some("anthropic"));
}

#[test]
fn completed_claude_stream_preserves_reported_completion_tokens() {
    let request = json!({"model":"claude-sonnet-4","messages":[{"role":"user","content":"hello"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::ClaudeChat, &request, "claude-sonnet-4");
    estimator
        .consume(b"data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"one two three four five six seven eight\"}}\n\n")
        .unwrap();

    let usage = estimator
        .apply_to_usage(
            Some(TokenUsage {
                prompt_tokens: Some(12),
                completion_tokens: Some(1),
                usage_source: Some("anthropic"),
                usage_semantic: Some("anthropic"),
                ..TokenUsage::default()
            }),
            true,
        )
        .expect("usage should stay present");

    assert_eq!(usage.completion_tokens, Some(1));
    assert_eq!(usage.usage_source, Some("anthropic"));
}

#[test]
fn estimates_gemini_text_delta_usage() {
    let request = json!({"model":"gemini-2.5-pro","contents":[{"role":"user","parts":[{"text":"hello"}]}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::GeminiChat, &request, "gemini-2.5-pro");
    estimator
        .consume(
            b"data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"world\"}]},\"finishReason\":\"FINISH_REASON_UNSPECIFIED\"}]}\n\n",
        )
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");

    assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
    assert_eq!(usage.usage_semantic, Some("gemini"));
    assert!(usage.prompt_tokens.unwrap() > 0);
    assert!(usage.completion_tokens.unwrap() > 0);
}

#[test]
fn gemini_cumulative_chunks_do_not_double_count_text() {
    let request = json!({"model":"gemini-2.5-pro","contents":[{"role":"user","parts":[{"text":"hello"}]}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::GeminiChat, &request, "gemini-2.5-pro");
    estimator
        .consume(b"data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"hello\"}]}}]}\n\n")
        .unwrap();
    estimator
        .consume(b"data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"hello world\"}]}}]}\n\n")
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");
    let expected = StreamUsageEstimator::new(ApiFormat::GeminiChat, &request, "gemini-2.5-pro")
        .apply_to_usage(None, false)
        .unwrap_or_default();
    let direct = crate::llm_proxy::proxy::stream_transport::token_estimator::estimate_text_tokens("gemini-2.5-pro", "hello world");

    assert_eq!(usage.completion_tokens, Some(direct));
    assert_ne!(usage.completion_tokens, expected.completion_tokens);
}

#[test]
fn estimates_gemini_inline_image_usage() {
    let request = json!({"model":"gemini-2.5-pro","contents":[{"role":"user","parts":[{"text":"draw"}]}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::GeminiChat, &request, "gemini-2.5-pro");
    estimator
        .consume(b"data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"inlineData\":{\"mimeType\":\"image/png\",\"data\":\"abc\"}}]}}]}\n\n")
        .unwrap();

    let usage = estimator.estimated_usage().expect("usage should be estimated");

    assert_eq!(usage.usage_semantic, Some("gemini"));
    assert_eq!(usage.output_image_tokens, Some(1400));
    assert_eq!(usage.completion_tokens, Some(1400));
}

#[test]
fn empty_output_does_not_estimate_usage() {
    let request = json!({"model":"gpt-5.5","input":[{"role":"user","content":"hello"}]});
    let estimator = StreamUsageEstimator::new(ApiFormat::OpenAiResponses, &request, "gpt-5.5");

    assert!(estimator.estimated_usage().is_none());
}

#[test]
fn empty_claude_output_does_not_estimate_usage() {
    let request = json!({"model":"claude-sonnet-4","messages":[{"role":"user","content":"hello"}]});
    let estimator = StreamUsageEstimator::new(ApiFormat::ClaudeChat, &request, "claude-sonnet-4");

    assert!(estimator.estimated_usage().is_none());
}

#[test]
fn empty_gemini_output_does_not_estimate_usage() {
    let request = json!({"model":"gemini-2.5-pro","contents":[{"role":"user","parts":[{"text":"hello"}]}]});
    let estimator = StreamUsageEstimator::new(ApiFormat::GeminiChat, &request, "gemini-2.5-pro");

    assert!(estimator.estimated_usage().is_none());
}

#[test]
fn finish_reads_last_line_without_trailing_newline() {
    let request = json!({"model":"gpt-5.5","input":[{"role":"user","content":"hello"}]});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::OpenAiResponses, &request, "gpt-5.5");
    estimator
        .consume(b"data: {\"type\":\"response.output_text.delta\",\"delta\":\"world\"}")
        .unwrap();
    estimator.finish().unwrap();

    assert!(estimator.estimated_usage().is_some());
}

#[test]
fn fills_missing_completion_tokens_without_overwriting_prompt_tokens() {
    let request = json!({"model":"gpt-5.5","input":"hello"});
    let mut estimator = StreamUsageEstimator::new(ApiFormat::OpenAiResponses, &request, "gpt-5.5");
    estimator
        .consume(b"data: {\"type\":\"response.output_text.delta\",\"delta\":\"world\"}\n\n")
        .unwrap();

    let usage = estimator
        .apply_to_usage(
            Some(TokenUsage {
                prompt_tokens: Some(42),
                usage_source: Some("openai"),
                usage_semantic: Some("responses"),
                ..TokenUsage::default()
            }),
            false,
        )
        .expect("usage should stay present");

    assert_eq!(usage.prompt_tokens, Some(42));
    assert!(usage.completion_tokens.unwrap() > 0);
    assert_eq!(usage.total_tokens, Some(42 + usage.completion_tokens.unwrap()));
    assert_eq!(usage.usage_source, Some("estimated_from_stream_delta"));
    assert_eq!(usage.usage_semantic, Some("responses"));
}
