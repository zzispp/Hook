use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};

use super::LlmProxyError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointFamily {
    OpenAi,
    Claude,
    Gemini,
    Rerank,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointKind {
    Chat,
    Completion,
    Responses,
    Compact,
    ImageGeneration,
    ImageEdit,
    Embedding,
    AudioTranscription,
    AudioTranslation,
    AudioSpeech,
    Moderation,
    Realtime,
    GeminiEmbedContent,
    GeminiBatchEmbedContents,
    GeminiVideo,
    Rerank,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthScheme {
    Bearer,
    Anthropic,
    Gemini,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpstreamStreamPolicy {
    MirrorClient,
    ForceNonStream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndpointMetadata {
    pub endpoint_id: &'static str,
    pub family: EndpointFamily,
    pub kind: EndpointKind,
    pub data_format: ApiFormat,
    pub default_path: &'static str,
    pub model_in_body: bool,
    pub stream_in_body: bool,
    pub auth_scheme: AuthScheme,
    pub upstream_stream_policy: UpstreamStreamPolicy,
    pub include_usage_for_stream: bool,
}

pub fn endpoint_metadata(format: &str, is_stream: bool) -> Result<EndpointMetadata, LlmProxyError> {
    let normalized = normalize_format_id(format);
    let metadata = match normalized.as_str() {
        "openai:chat" => openai_chat(),
        "openai_completion" | "openai_completions" | "openai:completion" | "openai:completions" => openai_completion(),
        "openai:cli" => openai_responses(),
        "openai:compact" => openai_compact(),
        "openai_image" | "openai_images" | "openai_images_generations" | "openai:image" | "openai:image_generation" => openai_image_generation(),
        "openai_image_edit" | "openai_images_edits" | "openai_edits" | "openai:image_edit" => openai_image_edit(),
        "openai_embedding" | "openai_embeddings" | "openai:embedding" | "openai:embeddings" => openai_embedding(),
        "openai_audio_transcription" | "openai_audio_transcriptions" | "openai:audio_transcription" => openai_audio_transcription(),
        "openai_audio_translation" | "openai_audio_translations" | "openai:audio_translation" => openai_audio_translation(),
        "openai_audio_speech" | "openai:audio_speech" => openai_audio_speech(),
        "openai_moderation" | "openai_moderations" | "openai:moderation" | "openai:moderations" => openai_moderation(),
        "openai_realtime" | "openai:realtime" => openai_realtime(),
        "claude:chat" => claude_chat(AuthScheme::Anthropic),
        "claude:cli" => claude_chat(AuthScheme::Bearer),
        "gemini:chat" | "gemini:cli" => gemini_chat(is_stream, normalized == "gemini:cli"),
        "gemini_embedding" | "gemini_embed_content" | "gemini:embedding" | "gemini:embed_content" => gemini_embedding(),
        "gemini_batch_embedding" | "gemini_batch_embed_contents" | "gemini:batch_embed_contents" => gemini_batch_embedding(),
        "gemini_video" | "gemini:video" | "veo" => gemini_video(),
        "rerank" | "jina_rerank" | "jina:rerank" => rerank(),
        _ => return Err(LlmProxyError::InvalidRequest(format!("unsupported API format: {format}"))),
    };
    Ok(metadata)
}

pub fn formats_compatible(client_format: &str, provider_format: &str, is_stream: bool) -> bool {
    let Ok(client) = endpoint_metadata(client_format, is_stream) else {
        return false;
    };
    let Ok(provider) = endpoint_metadata(provider_format, is_stream) else {
        return false;
    };
    if !stream_policy_compatible(provider, is_stream) {
        return false;
    }
    if endpoint_protocol_matches(client, provider) {
        return true;
    }
    if client.data_format == provider.data_format {
        return false;
    }
    if !cross_protocol_conversion_allowed(client, provider) {
        return false;
    }
    if !client.data_format.supports_chat_conversion() || !provider.data_format.supports_chat_conversion() {
        return false;
    }
    FormatConversionRegistry.can_convert(client.data_format, provider.data_format, is_stream)
}

pub fn formats_exact(client_format: &str, provider_format: &str, is_stream: bool) -> Result<bool, LlmProxyError> {
    let client = endpoint_metadata(client_format, is_stream)?;
    let provider = endpoint_metadata(provider_format, is_stream)?;
    Ok(stream_policy_compatible(provider, is_stream) && endpoint_protocol_matches(client, provider))
}

pub fn needs_conversion(client_format: &str, provider_format: &str, is_stream: bool) -> Result<bool, LlmProxyError> {
    let client = endpoint_metadata(client_format, is_stream)?;
    let provider = endpoint_metadata(provider_format, is_stream)?;
    Ok(client.data_format != provider.data_format)
}

pub fn conversion_formats(client_format: &str, provider_format: &str, is_stream: bool) -> Result<(ApiFormat, ApiFormat), LlmProxyError> {
    let client = endpoint_metadata(client_format, is_stream)?;
    let provider = endpoint_metadata(provider_format, is_stream)?;
    let source = client.data_format;
    let target = provider.data_format;
    if source == target || (cross_protocol_conversion_allowed(client, provider) && FormatConversionRegistry.can_convert(source, target, is_stream)) {
        return Ok((source, target));
    }
    Err(LlmProxyError::InvalidRequest(format!(
        "format conversion is not supported from {client_format} to {provider_format}"
    )))
}

pub fn render_path(format: &str, path: &str, model: &str, is_stream: bool) -> String {
    path.replace("{model}", model).replace("{action}", endpoint_action(format, is_stream))
}

fn endpoint_action(format: &str, is_stream: bool) -> &'static str {
    let Ok(metadata) = endpoint_metadata(format, is_stream) else {
        return "";
    };
    match (metadata.family, metadata.kind, is_stream) {
        (EndpointFamily::Gemini, EndpointKind::Chat, true) => "streamGenerateContent",
        (EndpointFamily::Gemini, EndpointKind::Chat, false) => "generateContent",
        _ => "",
    }
}

fn stream_policy_compatible(provider: EndpointMetadata, is_stream: bool) -> bool {
    !is_stream || provider.upstream_stream_policy != UpstreamStreamPolicy::ForceNonStream
}

fn endpoint_protocol_matches(client: EndpointMetadata, provider: EndpointMetadata) -> bool {
    client.family == provider.family && client.kind == provider.kind && client.data_format == provider.data_format
}

fn cross_protocol_conversion_allowed(client: EndpointMetadata, provider: EndpointMetadata) -> bool {
    if openai_conversion_requires_cli(client) && provider.family == EndpointFamily::OpenAi {
        return provider.endpoint_id == "openai:cli";
    }
    true
}

fn openai_conversion_requires_cli(client: EndpointMetadata) -> bool {
    matches!(client.endpoint_id, "claude:chat" | "gemini:chat" | "gemini:cli")
}

fn openai_chat() -> EndpointMetadata {
    openai_metadata("openai:chat", EndpointKind::Chat, ApiFormat::OpenAiChat, "/v1/chat/completions", true, true)
}

fn openai_completion() -> EndpointMetadata {
    openai_metadata(
        "openai_completion",
        EndpointKind::Completion,
        ApiFormat::OpenAiCompletion,
        "/v1/completions",
        true,
        false,
    )
}

fn openai_responses() -> EndpointMetadata {
    openai_metadata("openai:cli", EndpointKind::Responses, ApiFormat::OpenAiResponses, "/v1/responses", true, true)
}

fn openai_compact() -> EndpointMetadata {
    let mut metadata = openai_metadata(
        "openai:compact",
        EndpointKind::Compact,
        ApiFormat::OpenAiResponsesCompact,
        "/v1/responses/compact",
        true,
        false,
    );
    metadata.upstream_stream_policy = UpstreamStreamPolicy::ForceNonStream;
    metadata
}

fn openai_image_generation() -> EndpointMetadata {
    openai_metadata(
        "openai_image",
        EndpointKind::ImageGeneration,
        ApiFormat::OpenAiImage,
        "/v1/images/generations",
        true,
        true,
    )
}

fn openai_image_edit() -> EndpointMetadata {
    openai_metadata(
        "openai_image_edit",
        EndpointKind::ImageEdit,
        ApiFormat::OpenAiImage,
        "/v1/images/edits",
        true,
        true,
    )
}

fn openai_embedding() -> EndpointMetadata {
    openai_metadata(
        "openai_embedding",
        EndpointKind::Embedding,
        ApiFormat::OpenAiEmbedding,
        "/v1/embeddings",
        true,
        false,
    )
}

fn openai_audio_transcription() -> EndpointMetadata {
    openai_metadata(
        "openai_audio_transcription",
        EndpointKind::AudioTranscription,
        ApiFormat::OpenAiAudio,
        "/v1/audio/transcriptions",
        true,
        false,
    )
}

fn openai_audio_translation() -> EndpointMetadata {
    openai_metadata(
        "openai_audio_translation",
        EndpointKind::AudioTranslation,
        ApiFormat::OpenAiAudio,
        "/v1/audio/translations",
        true,
        false,
    )
}

fn openai_audio_speech() -> EndpointMetadata {
    openai_metadata(
        "openai_audio_speech",
        EndpointKind::AudioSpeech,
        ApiFormat::OpenAiAudio,
        "/v1/audio/speech",
        true,
        false,
    )
}

fn openai_moderation() -> EndpointMetadata {
    openai_metadata(
        "openai_moderation",
        EndpointKind::Moderation,
        ApiFormat::OpenAiModeration,
        "/v1/moderations",
        true,
        false,
    )
}

fn openai_realtime() -> EndpointMetadata {
    openai_metadata(
        "openai_realtime",
        EndpointKind::Realtime,
        ApiFormat::OpenAiRealtime,
        "/v1/realtime",
        false,
        false,
    )
}

fn openai_metadata(
    endpoint_id: &'static str,
    kind: EndpointKind,
    data_format: ApiFormat,
    default_path: &'static str,
    model_in_body: bool,
    stream_in_body: bool,
) -> EndpointMetadata {
    EndpointMetadata {
        endpoint_id,
        family: EndpointFamily::OpenAi,
        kind,
        data_format,
        default_path,
        model_in_body,
        stream_in_body,
        auth_scheme: AuthScheme::Bearer,
        upstream_stream_policy: UpstreamStreamPolicy::MirrorClient,
        include_usage_for_stream: matches!(
            data_format,
            ApiFormat::OpenAiChat | ApiFormat::OpenAiResponses | ApiFormat::OpenAiResponsesCompact
        ),
    }
}

fn claude_chat(auth_scheme: AuthScheme) -> EndpointMetadata {
    EndpointMetadata {
        endpoint_id: if auth_scheme == AuthScheme::Bearer { "claude:cli" } else { "claude:chat" },
        family: EndpointFamily::Claude,
        kind: EndpointKind::Chat,
        data_format: ApiFormat::ClaudeChat,
        default_path: "/v1/messages",
        model_in_body: true,
        stream_in_body: true,
        auth_scheme,
        upstream_stream_policy: UpstreamStreamPolicy::MirrorClient,
        include_usage_for_stream: false,
    }
}

fn gemini_chat(is_stream: bool, is_cli: bool) -> EndpointMetadata {
    EndpointMetadata {
        endpoint_id: if is_cli { "gemini:cli" } else { "gemini:chat" },
        family: EndpointFamily::Gemini,
        kind: EndpointKind::Chat,
        data_format: ApiFormat::GeminiChat,
        default_path: gemini_chat_path(is_stream),
        model_in_body: false,
        stream_in_body: false,
        auth_scheme: AuthScheme::Gemini,
        upstream_stream_policy: UpstreamStreamPolicy::MirrorClient,
        include_usage_for_stream: false,
    }
}

fn gemini_embedding() -> EndpointMetadata {
    gemini_metadata(
        "gemini_embedding",
        EndpointKind::GeminiEmbedContent,
        ApiFormat::GeminiEmbedding,
        "/v1beta/models/{model}:embedContent",
    )
}

fn gemini_batch_embedding() -> EndpointMetadata {
    gemini_metadata(
        "gemini_batch_embedding",
        EndpointKind::GeminiBatchEmbedContents,
        ApiFormat::GeminiEmbedding,
        "/v1beta/models/{model}:batchEmbedContents",
    )
}

fn gemini_video() -> EndpointMetadata {
    gemini_metadata(
        "gemini_video",
        EndpointKind::GeminiVideo,
        ApiFormat::GeminiVideo,
        "/v1beta/models/{model}:predictLongRunning",
    )
}

fn gemini_metadata(endpoint_id: &'static str, kind: EndpointKind, data_format: ApiFormat, default_path: &'static str) -> EndpointMetadata {
    EndpointMetadata {
        endpoint_id,
        family: EndpointFamily::Gemini,
        kind,
        data_format,
        default_path,
        model_in_body: false,
        stream_in_body: false,
        auth_scheme: AuthScheme::Gemini,
        upstream_stream_policy: UpstreamStreamPolicy::MirrorClient,
        include_usage_for_stream: false,
    }
}

fn rerank() -> EndpointMetadata {
    EndpointMetadata {
        endpoint_id: "rerank",
        family: EndpointFamily::Rerank,
        kind: EndpointKind::Rerank,
        data_format: ApiFormat::Rerank,
        default_path: "/v1/rerank",
        model_in_body: true,
        stream_in_body: false,
        auth_scheme: AuthScheme::Bearer,
        upstream_stream_policy: UpstreamStreamPolicy::MirrorClient,
        include_usage_for_stream: false,
    }
}

fn gemini_chat_path(is_stream: bool) -> &'static str {
    if is_stream {
        "/v1beta/models/{model}:{action}?alt=sse"
    } else {
        "/v1beta/models/{model}:{action}"
    }
}

fn normalize_format_id(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use proxy::format_conversion::ApiFormat;

    use super::{AuthScheme, EndpointFamily, EndpointKind, UpstreamStreamPolicy, endpoint_metadata};

    #[test]
    fn endpoint_metadata_describes_openai_chat_stream_usage_policy() {
        let metadata = endpoint_metadata("openai:chat", true).unwrap();

        assert_eq!(metadata.family, EndpointFamily::OpenAi);
        assert_eq!(metadata.kind, EndpointKind::Chat);
        assert_eq!(metadata.data_format, ApiFormat::OpenAiChat);
        assert!(metadata.model_in_body);
        assert!(metadata.stream_in_body);
        assert!(metadata.include_usage_for_stream);
        assert_eq!(metadata.auth_scheme, AuthScheme::Bearer);
        assert_eq!(metadata.upstream_stream_policy, UpstreamStreamPolicy::MirrorClient);
    }

    #[test]
    fn endpoint_metadata_describes_gemini_path_and_body_policy() {
        let metadata = endpoint_metadata("gemini:chat", true).unwrap();

        assert_eq!(metadata.default_path, "/v1beta/models/{model}:{action}?alt=sse");
        assert_eq!(metadata.data_format, ApiFormat::GeminiChat);
        assert!(!metadata.model_in_body);
        assert!(!metadata.stream_in_body);
        assert_eq!(metadata.auth_scheme, AuthScheme::Gemini);
    }

    #[test]
    fn endpoint_metadata_separates_non_chat_endpoint_identity_from_data_format() {
        let metadata = endpoint_metadata("openai_image", false).unwrap();

        assert_eq!(metadata.family, EndpointFamily::OpenAi);
        assert_eq!(metadata.kind, EndpointKind::ImageGeneration);
        assert_eq!(metadata.data_format, ApiFormat::OpenAiImage);
        assert_eq!(metadata.default_path, "/v1/images/generations");
        assert!(!metadata.include_usage_for_stream);
    }

    #[test]
    fn openai_image_metadata_preserves_stream_in_body() {
        for format in ["openai_image", "openai_image_edit"] {
            let metadata = endpoint_metadata(format, true).unwrap();

            assert!(metadata.stream_in_body, "{format} should forward stream to upstream");
            assert!(!metadata.include_usage_for_stream, "{format} should not add stream usage options");
            assert_eq!(metadata.upstream_stream_policy, UpstreamStreamPolicy::MirrorClient);
        }
    }

    #[test]
    fn streaming_requests_do_not_route_to_force_non_stream_formats() {
        assert!(!super::formats_compatible("openai:chat", "openai:compact", true));
        assert!(!super::formats_compatible("openai:cli", "openai:compact", true));
        assert!(super::formats_compatible("openai:chat", "openai:compact", false));
    }

    #[test]
    fn same_data_format_different_endpoint_kind_is_not_exact_or_compatible() {
        assert!(!super::formats_exact("openai:cli", "openai:compact", false).unwrap());
        assert!(!super::formats_compatible("openai:cli", "openai:compact", false));
        assert!(!super::formats_exact("openai_image", "openai_images_edits", false).unwrap());
        assert!(!super::formats_compatible("openai_image", "openai_images_edits", false));
    }

    #[test]
    fn cli_and_chat_same_data_format_remain_exact() {
        assert!(super::formats_exact("claude:chat", "claude:cli", true).unwrap());
        assert!(super::formats_compatible("claude:chat", "claude:cli", true));
    }

    #[test]
    fn responses_requests_route_to_chat_endpoints_through_internal() {
        assert!(super::formats_compatible("openai:cli", "openai:chat", true));
        assert!(super::formats_compatible("openai:cli", "openai:chat", false));
        assert!(super::formats_compatible("openai:chat", "openai:cli", true));
    }

    #[test]
    fn claude_and_gemini_chat_convert_only_to_openai_cli() {
        assert!(super::formats_compatible("claude:chat", "openai:cli", true));
        assert!(!super::formats_compatible("claude:chat", "openai:chat", true));
        assert!(super::conversion_formats("claude:chat", "openai:cli", true).is_ok());
        assert!(super::conversion_formats("claude:chat", "openai:chat", true).is_err());

        assert!(super::formats_compatible("gemini:chat", "openai:cli", true));
        assert!(!super::formats_compatible("gemini:chat", "openai:chat", true));
        assert!(super::conversion_formats("gemini:chat", "openai:cli", true).is_ok());
        assert!(super::conversion_formats("gemini:chat", "openai:chat", true).is_err());

        assert!(super::formats_compatible("gemini:cli", "openai:cli", true));
        assert!(!super::formats_compatible("gemini:cli", "openai:chat", true));
        assert!(super::conversion_formats("gemini:cli", "openai:cli", true).is_ok());
        assert!(super::conversion_formats("gemini:cli", "openai:chat", true).is_err());
    }

    #[test]
    fn non_chat_endpoints_never_convert_through_standard_formats() {
        assert!(!super::formats_compatible("openai:chat", "openai_image", false));
        assert!(!super::formats_compatible("openai_image", "openai:chat", false));
        assert!(!super::needs_conversion("openai_image", "openai_images_edits", false).unwrap());
    }
}
