mod audit;
mod auth;
mod billing;
mod cache;
mod cache_affinity;
mod candidate;
mod client_error;
mod codex_chat_history;
mod error;
mod formats;
mod handlers;
mod model_access;
mod model_test;
mod proxy;
mod rate_limit;
mod request_payload_writer;
mod request_record_policy;
pub(crate) mod routing;
mod token_usage;
mod ws;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};
use provider::infra::ProviderKeyCipher;
use req::ReqwestClient;
use storage::Database;
use types::wallet::Wallet;
use user::application::SystemUserProvider;

pub(crate) use cache::snapshot::CachedUserAccess;
pub use cache::{LlmProxyCache, LlmProxyCacheOptions};
pub(crate) use cache_affinity::{AffinityEntry, AffinityRecord, AffinitySelection, ClearAffinityInput, InvalidateAffinityInput, SetAffinityInput};
pub(crate) use candidate::routing_rankings;
pub use error::LlmProxyError;
pub(crate) use model_test::LlmProxyProviderModelTester;
pub(crate) use proxy::{ProxyJsonRequest, proxy_json};
pub(crate) use rate_limit::ProviderKeyProbeSlotOptions;

pub const OPENAI_CHAT_FORMAT: &str = "openai:chat";
pub const OPENAI_COMPLETION_FORMAT: &str = "openai_completion";
pub const OPENAI_CLI_FORMAT: &str = "openai:cli";
pub const OPENAI_COMPACT_FORMAT: &str = "openai:compact";
pub const OPENAI_IMAGE_FORMAT: &str = "openai_image";
pub const OPENAI_IMAGE_EDIT_FORMAT: &str = "openai_image_edit";
pub const IMAGE_GENERATION_CAPABILITY: &str = "image_generation";
pub const OPENAI_EMBEDDING_FORMAT: &str = "openai_embedding";
pub const OPENAI_AUDIO_TRANSCRIPTION_FORMAT: &str = "openai_audio_transcription";
pub const OPENAI_AUDIO_TRANSLATION_FORMAT: &str = "openai_audio_translation";
pub const OPENAI_AUDIO_SPEECH_FORMAT: &str = "openai_audio_speech";
pub const OPENAI_MODERATION_FORMAT: &str = "openai_moderation";
pub const CLAUDE_CHAT_FORMAT: &str = "claude:chat";
pub const GEMINI_CHAT_FORMAT: &str = "gemini:chat";
pub const GEMINI_EMBEDDING_FORMAT: &str = "gemini_embedding";
pub const GEMINI_BATCH_EMBEDDING_FORMAT: &str = "gemini_batch_embedding";
pub const RERANK_FORMAT: &str = "rerank";

pub const REALTIME_PATH: &str = "/v1/realtime";

pub(crate) fn cached_system_user_access(provider: &impl SystemUserProvider) -> Vec<CachedUserAccess> {
    provider.system_user().map(|record| cached_user_access(record.user)).into_iter().collect()
}

fn cached_user_access(user: types::user::User) -> CachedUserAccess {
    CachedUserAccess {
        id: user.id.0,
        username: user.username,
        group_codes: user.group_codes,
        is_active: user.is_active,
        allowed_model_ids: user.allowed_model_ids,
        allowed_provider_ids: user.allowed_provider_ids,
        quota_mode: user.quota_mode,
        rate_limit_rpm: user.rate_limit_rpm,
    }
}

#[derive(Clone)]
pub struct LlmProxyState {
    database: Database,
    cipher: ProviderKeyCipher,
    http: ReqwestClient,
    affinity: redis::aio::ConnectionManager,
    cache: LlmProxyCache,
    routing_metrics: routing::RoutingMetricsCache,
    codex_chat_history: codex_chat_history::CodexChatHistoryStore,
    payload_writer: request_payload_writer::RequestPayloadWriter,
    key_prefix: String,
    system_wallet: Option<Wallet>,
}

impl LlmProxyState {
    pub fn new(
        database: Database,
        cipher: ProviderKeyCipher,
        affinity: redis::aio::ConnectionManager,
        cache: LlmProxyCache,
        routing_metrics: routing::RoutingMetricsCache,
        key_prefix: String,
        system_wallet: Option<Wallet>,
    ) -> Self {
        let payload_writer = request_payload_writer::RequestPayloadWriter::spawn(database.clone());
        Self {
            database,
            cipher,
            http: llm_proxy_http_client(),
            affinity: affinity.clone(),
            cache,
            routing_metrics,
            codex_chat_history: codex_chat_history::CodexChatHistoryStore::new(affinity.clone(), key_prefix.clone()),
            payload_writer,
            key_prefix,
            system_wallet,
        }
    }

    pub async fn cached_api_token_by_hash(&self, token_hash: &str) -> Result<Option<types::api_token::ApiToken>, LlmProxyError> {
        self.cache.api_token_by_hash(token_hash).await
    }

    pub async fn scheduling_snapshot(&self) -> Result<cache::snapshot::SchedulingSnapshot, LlmProxyError> {
        self.cache.scheduling_snapshot().await
    }

    pub(crate) async fn routing_metrics_snapshot(&self) -> routing::RoutingMetricsSnapshot {
        self.routing_metrics.snapshot().await
    }

    pub(crate) fn codex_chat_history(&self) -> &codex_chat_history::CodexChatHistoryStore {
        &self.codex_chat_history
    }

    pub(crate) fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn system_wallet_for_user(&self, user_id: &str) -> Option<Wallet> {
        self.system_wallet.as_ref().filter(|wallet| wallet.user_id == user_id).cloned()
    }

    pub async fn cooled_provider_ids(&self, provider_ids: &[String]) -> Result<std::collections::HashSet<String>, LlmProxyError> {
        self.cache.cooled_provider_ids(provider_ids).await
    }

    pub async fn record_provider_status_failure(&self, input: cache::ProviderCooldownFailureInput<'_>) -> Result<bool, LlmProxyError> {
        self.cache.record_provider_status_failure(input).await
    }

    pub async fn cached_affinity(&self, token_id: &str, model_id: &str, api_format: &str) -> Result<Option<AffinityRecord>, LlmProxyError> {
        self.affinity_store().get(token_id, model_id, api_format).await
    }

    pub async fn remember_affinity(&self, input: SetAffinityInput<'_>) -> Result<(), LlmProxyError> {
        self.affinity_store().set(input).await
    }

    pub async fn invalidate_affinity(&self, input: InvalidateAffinityInput<'_>) -> Result<(), LlmProxyError> {
        self.affinity_store().invalidate(input).await
    }

    pub async fn list_affinities(&self) -> Result<Vec<AffinityEntry>, LlmProxyError> {
        self.affinity_store().list().await
    }

    pub async fn clear_single_affinity(&self, input: ClearAffinityInput<'_>) -> Result<bool, LlmProxyError> {
        let Some(record) = self.cached_affinity(input.token_id, input.model_id, input.api_format).await? else {
            return Ok(false);
        };
        let Some(invalidate_input) = exact_invalidate_input(&record, input) else {
            return Ok(false);
        };
        self.invalidate_affinity(invalidate_input).await?;
        Ok(true)
    }

    pub async fn clear_all_affinities(&self) -> Result<u64, LlmProxyError> {
        self.affinity_store().clear_all().await
    }

    fn affinity_store(&self) -> cache_affinity::CacheAffinityStore {
        cache_affinity::CacheAffinityStore::new(self.affinity.clone(), &self.key_prefix)
    }

    async fn enqueue_request_payload(&self, job: request_payload_writer::RequestPayloadJob) -> Result<(), LlmProxyError> {
        self.payload_writer.enqueue(self.database.clone(), job).await
    }
}

fn exact_invalidate_input<'a>(record: &'a AffinityRecord, input: ClearAffinityInput<'a>) -> Option<InvalidateAffinityInput<'a>> {
    if record.endpoint_id != input.endpoint_id {
        return None;
    }
    Some(InvalidateAffinityInput {
        token_id: input.token_id,
        model_id: input.model_id,
        api_format: input.api_format,
        provider_id: record.provider_id.as_str(),
        endpoint_id: record.endpoint_id.as_str(),
        key_id: record.key_id.as_str(),
    })
}

fn llm_proxy_http_client() -> ReqwestClient {
    ReqwestClient::from_builder(req::long_stream_builder()).expect("LLM proxy req client builder should be valid")
}

pub fn create_router(state: LlmProxyState) -> Router {
    with_llm_proxy_body_limit(
        Router::new()
            .route("/models", get(handlers::list_models))
            .route("/models/", get(handlers::list_models))
            .route("/models/{model}", get(handlers::retrieve_model))
            .route("/models/{model}/", get(handlers::retrieve_model))
            .route("/usage", get(token_usage::usage))
            .route("/usage/", get(token_usage::usage))
            .route("/completions", post(handlers::completions))
            .route("/completions/", post(handlers::completions))
            .route("/chat/completions", post(handlers::chat_completions))
            .route("/chat/completions/", post(handlers::chat_completions))
            .route("/responses", post(handlers::responses))
            .route("/responses/", post(handlers::responses))
            .route("/responses/compact", post(handlers::responses_compact))
            .route("/responses/compact/", post(handlers::responses_compact))
            .route("/images/generations", post(handlers::image_generations))
            .route("/images/generations/", post(handlers::image_generations))
            .route("/images/edits", post(handlers::image_edits))
            .route("/images/edits/", post(handlers::image_edits))
            .route("/images/variations", post(handlers::image_variations))
            .route("/images/variations/", post(handlers::image_variations))
            .route("/edits", post(handlers::image_edits))
            .route("/edits/", post(handlers::image_edits))
            .route("/embeddings", post(handlers::embeddings))
            .route("/embeddings/", post(handlers::embeddings))
            .route("/audio/transcriptions", post(handlers::audio_transcriptions))
            .route("/audio/transcriptions/", post(handlers::audio_transcriptions))
            .route("/audio/translations", post(handlers::audio_translations))
            .route("/audio/translations/", post(handlers::audio_translations))
            .route("/audio/speech", post(handlers::audio_speech))
            .route("/audio/speech/", post(handlers::audio_speech))
            .route("/moderations", post(handlers::moderations))
            .route("/moderations/", post(handlers::moderations))
            .route("/rerank", post(handlers::rerank))
            .route("/rerank/", post(handlers::rerank))
            .route("/messages", post(handlers::claude_messages))
            .route("/messages/", post(handlers::claude_messages))
            .route("/realtime", get(ws::realtime))
            .route("/realtime/", get(ws::realtime))
            .with_state(state.clone())
            .layer(middleware::from_fn_with_state(state, auth::token_middleware)),
    )
}

pub fn create_v1beta_router(state: LlmProxyState) -> Router {
    with_llm_proxy_body_limit(
        Router::new()
            .route("/models/{model_action}", post(handlers::gemini_generate_content))
            .route("/models/{model_action}/", post(handlers::gemini_generate_content))
            .route("/models/{model}/embedContent", post(handlers::gemini_embed_content))
            .route("/models/{model}/embedContent/", post(handlers::gemini_embed_content))
            .route("/models/{model}/batchEmbedContents", post(handlers::gemini_batch_embed_contents))
            .route("/models/{model}/batchEmbedContents/", post(handlers::gemini_batch_embed_contents))
            .with_state(state.clone())
            .layer(middleware::from_fn_with_state(state, auth::token_middleware)),
    )
}

fn with_llm_proxy_body_limit(router: Router) -> Router {
    router.layer(DefaultBodyLimit::disable())
}

#[derive(Clone)]
pub struct CurrentApiToken(pub types::api_token::ApiToken);

#[cfg(test)]
mod tests {
    use axum::{
        Json, Router,
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode, header},
        routing::post,
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::{AffinityRecord, ClearAffinityInput, exact_invalidate_input};

    const OVERSIZED_JSON_BYTES: usize = 2_097_153;

    #[tokio::test]
    async fn llm_proxy_body_limit_allows_json_larger_than_axum_default() {
        let app = super::with_llm_proxy_body_limit(Router::new().route("/chat/completions", post(echo_body_size)));
        let response = app.oneshot(json_request("/chat/completions", oversized_json())).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_json(response).await["size"], OVERSIZED_JSON_BYTES);
    }

    async fn echo_body_size(Json(body): Json<Value>) -> Json<Value> {
        Json(json!({ "size": body.to_string().len() }))
    }

    fn oversized_json() -> Value {
        json!({ "input": "x".repeat(OVERSIZED_JSON_BYTES - r#"{"input":""}"#.len()) })
    }

    fn json_request(uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn response_json(response: axum::response::Response) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[test]
    fn exact_invalidate_input_requires_same_endpoint() {
        let record = AffinityRecord {
            provider_id: "provider-1".into(),
            endpoint_id: "endpoint-1".into(),
            key_id: "key-1".into(),
            api_format: "openai:chat".into(),
            model_id: "model-1".into(),
            created_at: 0,
            expire_at: 60,
            request_count: 1,
        };

        let matched = exact_invalidate_input(
            &record,
            ClearAffinityInput {
                token_id: "token-1",
                model_id: "model-1",
                api_format: "openai:chat",
                endpoint_id: "endpoint-1",
            },
        )
        .unwrap();
        assert_eq!(matched.key_id, "key-1");

        let mismatched = exact_invalidate_input(
            &record,
            ClearAffinityInput {
                token_id: "token-1",
                model_id: "model-1",
                api_format: "openai:chat",
                endpoint_id: "endpoint-2",
            },
        );
        assert!(mismatched.is_none());
    }
}
