use proxy::format_conversion::ApiFormat;
use serde_json::Value;

use super::{LlmProxyError, request::AttemptContext};
use crate::llm_proxy::codex_chat_history::CodexChatHistoryError;

pub(super) async fn enrich_responses_chat_request(
    context: AttemptContext<'_>,
    body: &mut Value,
    source: ApiFormat,
    target: ApiFormat,
) -> Result<(), LlmProxyError> {
    if !matches!(source, ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact) || target != ApiFormat::OpenAiChat {
        return Ok(());
    }
    context.codex_chat_history.enrich_request(body).await.map(|_| ()).map_err(history_error)
}

fn history_error(error: CodexChatHistoryError) -> LlmProxyError {
    match error {
        CodexChatHistoryError::Infrastructure(message) => LlmProxyError::Infrastructure(message),
        CodexChatHistoryError::Missing { .. } | CodexChatHistoryError::Ambiguous { .. } => LlmProxyError::InvalidRequest(error.to_string()),
    }
}

#[cfg(test)]
#[path = "request_codex_history_tests.rs"]
mod tests;
