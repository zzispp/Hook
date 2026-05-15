mod audit;
mod auth;
mod billing;
mod cache;
mod candidate;
mod error;
mod formats;
mod handlers;
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

pub use cache::LlmProxyCache;
pub use error::LlmProxyError;

pub const OPENAI_CHAT_FORMAT: &str = "openai_chat";
pub const OPENAI_CLI_FORMAT: &str = "openai_cli";
pub const OPENAI_COMPACT_FORMAT: &str = "openai_compact";
pub const CLAUDE_CHAT_FORMAT: &str = "claude_chat";
pub const GEMINI_CHAT_FORMAT: &str = "gemini_chat";

pub const REALTIME_PATH: &str = "/v1/realtime";

#[derive(Clone)]
pub struct LlmProxyState {
    database: Database,
    cipher: ProviderKeyCipher,
    http: ReqwestClient,
    affinity: redis::aio::ConnectionManager,
    cache: LlmProxyCache,
    key_prefix: String,
}

impl LlmProxyState {
    pub fn new(database: Database, cipher: ProviderKeyCipher, affinity: redis::aio::ConnectionManager, cache: LlmProxyCache, key_prefix: String) -> Self {
        Self {
            database,
            cipher,
            http: ReqwestClient::default(),
            affinity,
            cache,
            key_prefix,
        }
    }

    pub async fn cached_api_token_by_hash(&self, token_hash: &str) -> Result<Option<types::api_token::ApiToken>, LlmProxyError> {
        self.cache.api_token_by_hash(token_hash).await
    }

    pub async fn scheduling_snapshot(&self) -> Result<cache::snapshot::SchedulingSnapshot, LlmProxyError> {
        self.cache.scheduling_snapshot().await
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
        .route("/chat/completions", post(handlers::chat_completions))
        .route("/chat/completions/", post(handlers::chat_completions))
        .route("/responses", post(handlers::responses))
        .route("/responses/", post(handlers::responses))
        .route("/responses/compact", post(handlers::responses_compact))
        .route("/responses/compact/", post(handlers::responses_compact))
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
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, auth::token_middleware))
}

#[derive(Clone)]
pub struct CurrentApiToken(pub types::api_token::ApiToken);
