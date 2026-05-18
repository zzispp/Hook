mod audit;
mod auth;
mod billing;
mod cache;
mod candidate;
mod client_error;
mod error;
mod formats;
mod handlers;
mod model_access;
mod model_test;
mod proxy;
mod rate_limit;
mod request_record_policy;
mod ws;

use axum::{
    Router, middleware,
    routing::{get, post},
};
use provider::infra::ProviderKeyCipher;
use redis::AsyncCommands;
use req::ReqwestClient;
use storage::Database;
use types::wallet::Wallet;
use user::application::SystemUserProvider;

pub use cache::LlmProxyCache;
pub(crate) use cache::snapshot::CachedUserAccess;
pub use error::LlmProxyError;
pub(crate) use model_test::LlmProxyProviderModelTester;

pub const OPENAI_CHAT_FORMAT: &str = "openai_chat";
pub const OPENAI_COMPLETION_FORMAT: &str = "openai_completion";
pub const OPENAI_CLI_FORMAT: &str = "openai_cli";
pub const OPENAI_COMPACT_FORMAT: &str = "openai_compact";
pub const OPENAI_IMAGE_FORMAT: &str = "openai_image";
pub const OPENAI_IMAGE_EDIT_FORMAT: &str = "openai_image_edit";
pub const OPENAI_EMBEDDING_FORMAT: &str = "openai_embedding";
pub const OPENAI_AUDIO_TRANSCRIPTION_FORMAT: &str = "openai_audio_transcription";
pub const OPENAI_AUDIO_TRANSLATION_FORMAT: &str = "openai_audio_translation";
pub const OPENAI_AUDIO_SPEECH_FORMAT: &str = "openai_audio_speech";
pub const OPENAI_MODERATION_FORMAT: &str = "openai_moderation";
pub const CLAUDE_CHAT_FORMAT: &str = "claude_chat";
pub const GEMINI_CHAT_FORMAT: &str = "gemini_chat";
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
    key_prefix: String,
    system_wallet: Option<Wallet>,
}

impl LlmProxyState {
    pub fn new(
        database: Database,
        cipher: ProviderKeyCipher,
        affinity: redis::aio::ConnectionManager,
        cache: LlmProxyCache,
        key_prefix: String,
        system_wallet: Option<Wallet>,
    ) -> Self {
        Self {
            database,
            cipher,
            http: ReqwestClient::default(),
            affinity,
            cache,
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

    pub fn system_wallet_for_user(&self, user_id: &str) -> Option<Wallet> {
        self.system_wallet.as_ref().filter(|wallet| wallet.user_id == user_id).cloned()
    }

    pub async fn cooled_provider_ids(&self, provider_ids: &[String]) -> Result<std::collections::HashSet<String>, LlmProxyError> {
        self.cache.cooled_provider_ids(provider_ids).await
    }

    pub async fn record_provider_status_failure(&self, input: cache::ProviderCooldownFailureInput<'_>) -> Result<bool, LlmProxyError> {
        self.cache.record_provider_status_failure(input).await
    }

    pub async fn cached_affinity_key(&self, token_id: &str, model_id: &str, api_format: &str) -> Result<Option<String>, LlmProxyError> {
        let mut connection = self.affinity.clone();
        connection
            .get(self.affinity_cache_key(token_id, model_id, api_format))
            .await
            .map_err(redis_error)
    }

    pub async fn remember_affinity_key(&self, token_id: &str, model_id: &str, api_format: &str, key_id: &str, ttl_minutes: i32) -> Result<(), LlmProxyError> {
        if ttl_minutes <= 0 {
            return Ok(());
        }
        let mut connection = self.affinity.clone();
        let seconds = ttl_minutes as u64 * 60;
        connection
            .set_ex(self.affinity_cache_key(token_id, model_id, api_format), key_id, seconds)
            .await
            .map_err(redis_error)
    }

    fn affinity_cache_key(&self, token_id: &str, model_id: &str, api_format: &str) -> String {
        format!("{}:llm_proxy:affinity:{token_id}:{model_id}:{api_format}", self.key_prefix)
    }
}

fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(error.to_string())
}

pub fn create_router(state: LlmProxyState) -> Router {
    Router::new()
        .route("/models", get(handlers::list_models))
        .route("/models/", get(handlers::list_models))
        .route("/models/{model}", get(handlers::retrieve_model))
        .route("/models/{model}/", get(handlers::retrieve_model))
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
        .layer(middleware::from_fn_with_state(state, auth::token_middleware))
}

pub fn create_v1beta_router(state: LlmProxyState) -> Router {
    Router::new()
        .route("/models/{model_action}", post(handlers::gemini_generate_content))
        .route("/models/{model_action}/", post(handlers::gemini_generate_content))
        .route("/models/{model}/embedContent", post(handlers::gemini_embed_content))
        .route("/models/{model}/embedContent/", post(handlers::gemini_embed_content))
        .route("/models/{model}/batchEmbedContents", post(handlers::gemini_batch_embed_contents))
        .route("/models/{model}/batchEmbedContents/", post(handlers::gemini_batch_embed_contents))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, auth::token_middleware))
}

#[derive(Clone)]
pub struct CurrentApiToken(pub types::api_token::ApiToken);
