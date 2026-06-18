use proxy::format_conversion::ApiFormat;

use crate::llm_proxy::{LlmProxyError, codex_chat_history::CodexChatHistoryStore};

pub(super) async fn record_non_stream_response(history: &CodexChatHistoryStore, source_format: ApiFormat, body: &[u8]) -> Result<usize, LlmProxyError> {
    if !matches!(source_format, ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact) {
        return Ok(0);
    }
    let response = serde_json::from_slice(body).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))?;
    history.record_response(&response).await
}

#[cfg(test)]
mod tests {
    use proxy::format_conversion::ApiFormat;
    use serde_json::json;

    use super::record_non_stream_response;
    use crate::llm_proxy::codex_chat_history::test_support::test_history;

    #[tokio::test]
    async fn records_openai_responses_body() {
        let history = test_history().await;
        let body = json!({
            "id": "resp_1",
            "output": [{
                "type": "function_call",
                "call_id": "call_1",
                "name": "read_file",
                "arguments": "{}"
            }]
        });

        let recorded = record_non_stream_response(&history, ApiFormat::OpenAiResponses, body.to_string().as_bytes())
            .await
            .unwrap();

        assert_eq!(recorded, 1);
    }

    #[tokio::test]
    async fn ignores_openai_chat_body() {
        let history = test_history().await;
        let body = json!({"id": "chatcmpl_1"});

        let recorded = record_non_stream_response(&history, ApiFormat::OpenAiChat, body.to_string().as_bytes())
            .await
            .unwrap();

        assert_eq!(recorded, 0);
    }
}
