use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};

use super::LlmProxyError;

pub fn parse_api_format(value: &str) -> Result<ApiFormat, LlmProxyError> {
    ApiFormat::parse(value).map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))
}

pub fn formats_compatible(client_format: &str, provider_format: &str, is_stream: bool) -> bool {
    let Ok(client) = ApiFormat::parse(client_format) else {
        return false;
    };
    let Ok(provider) = ApiFormat::parse(provider_format) else {
        return false;
    };
    client == provider || FormatConversionRegistry::default().can_convert(client, provider, is_stream)
}

pub fn default_path(format: &str, is_stream: bool) -> &'static str {
    match format {
        "openai_cli" => "/v1/responses",
        "openai_compact" => "/v1/responses/compact",
        "gemini_chat" | "gemini_cli" if is_stream => "/v1beta/models/{model}:{action}?alt=sse",
        "gemini_chat" | "gemini_cli" => "/v1beta/models/{model}:{action}",
        "claude_chat" | "claude_messages" => "/v1/messages",
        _ => "/v1/chat/completions",
    }
}

pub fn render_path(format: &str, path: &str, model: &str, is_stream: bool) -> String {
    path.replace("{model}", model).replace("{action}", endpoint_action(format, is_stream))
}

fn endpoint_action(format: &str, is_stream: bool) -> &'static str {
    match (format, is_stream) {
        ("gemini_chat" | "gemini_cli", true) => "streamGenerateContent",
        ("gemini_chat" | "gemini_cli", false) => "generateContent",
        _ => "",
    }
}
