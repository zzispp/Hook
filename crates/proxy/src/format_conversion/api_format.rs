use super::FormatConversionError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApiFormat {
    OpenAiChat,
    OpenAiResponses,
    GeminiChat,
    ClaudeChat,
}

impl ApiFormat {
    pub fn parse(value: &str) -> Result<Self, FormatConversionError> {
        let normalized = normalize_format_id(value);
        match normalized.as_str() {
            "openai" | "openaichat" => Ok(Self::OpenAiChat),
            "openairesponses" | "openaicli" | "openaicompact" => Ok(Self::OpenAiResponses),
            "gemini" | "geminichat" | "geminicli" => Ok(Self::GeminiChat),
            "claude" | "claudechat" | "claudecli" | "claudemessages" => Ok(Self::ClaudeChat),
            _ => Err(FormatConversionError::InvalidFormat(value.to_owned())),
        }
    }
}

fn normalize_format_id(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !matches!(ch, ':' | '_' | '-' | ' '))
        .collect()
}
