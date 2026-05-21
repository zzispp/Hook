use super::FormatConversionError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApiFormat {
    OpenAiChat,
    OpenAiCompletion,
    OpenAiResponses,
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
            "openai:cli" | "openai:compact" => Ok(Self::OpenAiResponses),
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
            "gemini:chat" | "gemini:cli" => Ok(Self::GeminiChat),
            "geminiembedding" | "geminiembeddings" | "geminiembedcontent" | "geminibatchembedcontents" => Ok(Self::GeminiEmbedding),
            "geminivideo" | "veo" => Ok(Self::GeminiVideo),
            "claude:chat" | "claude:cli" => Ok(Self::ClaudeChat),
            "rerank" | "jinarerank" => Ok(Self::Rerank),
            _ => Err(FormatConversionError::InvalidFormat(value.to_owned())),
        }
    }

    pub fn supports_chat_conversion(self) -> bool {
        matches!(self, Self::OpenAiChat | Self::OpenAiResponses | Self::GeminiChat | Self::ClaudeChat)
    }
}

fn normalize_format_id(value: &str) -> String {
    value.trim().to_ascii_lowercase().to_owned()
}
