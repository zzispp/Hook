use serde_json::Value;

pub use formats::{
    FormatContext, FormatError, FormatFamily, FormatId, FormatProfile, RequestConversionKind, SyncChatResponseConversionKind, SyncCliResponseConversionKind,
    is_embedding_api_format, is_openai_responses_compact_format, is_openai_responses_family_format, is_openai_responses_format, is_rerank_api_format,
    normalize_api_format_alias, request_candidate_api_format_preference, request_candidate_api_formats, request_conversion_kind,
    request_conversion_requires_enable_flag, sync_chat_response_conversion_kind, sync_cli_response_conversion_kind,
};

pub fn convert_request(body: &Value, source_format: &str, target_format: &str, context: &FormatContext) -> Result<Value, FormatError> {
    formats::convert_request(source_format, target_format, body, context)
}

pub fn convert_response(body: &Value, source_format: &str, target_format: &str, context: &FormatContext) -> Result<Value, FormatError> {
    formats::convert_response(source_format, target_format, body, context)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{FormatContext, convert_request};

    #[test]
    fn proxy_format_engine_converts_through_formats_crate() {
        let request = json!({
            "model": "gpt-source",
            "messages": [{ "role": "user", "content": "hello" }]
        });

        let converted = convert_request(
            &request,
            "openai:chat",
            "claude:messages",
            &FormatContext::default().with_mapped_model("claude-target"),
        )
        .expect("request conversion should succeed");

        assert_eq!(converted["model"], "claude-target");
        assert_eq!(converted["messages"][0]["role"], "user");
        assert_eq!(converted["messages"][0]["content"], "hello");
    }
}
