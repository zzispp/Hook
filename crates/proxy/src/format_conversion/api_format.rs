use super::FormatConversionError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApiFormat {
    OpenAiChat,
    OpenAiCompletion,
    OpenAiResponses,
    OpenAiResponsesCompact,
    OpenAiImage,
    OpenAiEmbedding,
    OpenAiAudio,
    OpenAiModeration,
    OpenAiRealtime,
    GeminiChat,
    GeminiEmbedding,
    GeminiVideo,
    ClaudeChat,
    Rerank,
}

impl ApiFormat {
    pub fn parse(value: &str) -> Result<Self, FormatConversionError> {
        let normalized = normalize_format_id(value);
        match normalized.as_str() {
            "openai:chat" => Ok(Self::OpenAiChat),
            "openaicompletion" | "openaicompletions" | "completion" | "completions" => Ok(Self::OpenAiCompletion),
            "openai:cli" | "openai:responses" => Ok(Self::OpenAiResponses),
            "openai:compact" | "openai:responses:compact" => Ok(Self::OpenAiResponsesCompact),
            "openaiimage"
            | "openaiimages"
            | "openaiimagegeneration"
            | "openaiimagesgeneration"
            | "openaiimagesgenerations"
            | "openaiimageedit"
            | "openaiimageedits"
            | "openaiimagesedit"
            | "openaiimagesedits"
            | "openaiedits"
            | "images"
            | "image"
            | "edits" => Ok(Self::OpenAiImage),
            "openaiembedding" | "openaiembeddings" | "embedding" | "embeddings" => Ok(Self::OpenAiEmbedding),
            "openaiaudio"
            | "openaiaudiotranscription"
            | "openaiaudiotranscriptions"
            | "openaiaudiotranslation"
            | "openaiaudiotranslations"
            | "openaiaudiospeech"
            | "audio" => Ok(Self::OpenAiAudio),
            "openaimoderation" | "openaimoderations" | "moderation" | "moderations" => Ok(Self::OpenAiModeration),
            "openairealtime" | "realtime" => Ok(Self::OpenAiRealtime),
            "gemini:chat" | "gemini:cli" | "gemini:generate_content" => Ok(Self::GeminiChat),
            "geminiembedding" | "geminiembeddings" | "geminiembedcontent" | "geminibatchembedcontents" => Ok(Self::GeminiEmbedding),
            "geminivideo" | "veo" => Ok(Self::GeminiVideo),
            "claude:chat" | "claude:cli" | "claude:messages" => Ok(Self::ClaudeChat),
            "rerank" | "jinarerank" => Ok(Self::Rerank),
            _ => Err(FormatConversionError::InvalidFormat(value.to_owned())),
        }
    }

    pub fn supports_chat_conversion(self) -> bool {
        matches!(
            self,
            Self::OpenAiChat | Self::OpenAiResponses | Self::OpenAiResponsesCompact | Self::GeminiChat | Self::ClaudeChat
        )
    }

    pub fn as_format_id(self) -> Result<&'static str, FormatConversionError> {
        match self {
            Self::OpenAiChat => Ok("openai:chat"),
            Self::OpenAiResponses => Ok("openai:responses"),
            Self::OpenAiResponsesCompact => Ok("openai:responses:compact"),
            Self::OpenAiEmbedding => Ok("openai:embedding"),
            Self::GeminiChat => Ok("gemini:generate_content"),
            Self::GeminiEmbedding => Ok("gemini:embedding"),
            Self::ClaudeChat => Ok("claude:messages"),
            Self::Rerank => Ok("openai:rerank"),
            _ => Err(FormatConversionError::unsupported_feature(
                "format_conversion",
                format!("format {self:?} is not supported by formats crate conversion"),
            )),
        }
    }

    pub fn is_stream_convertible(self) -> bool {
        matches!(
            self,
            Self::OpenAiChat | Self::OpenAiResponses | Self::OpenAiResponsesCompact | Self::GeminiChat | Self::ClaudeChat
        )
    }
}

fn normalize_format_id(value: &str) -> String {
    value.trim().to_ascii_lowercase().to_owned()
}
