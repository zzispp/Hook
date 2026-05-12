use std::sync::Arc;

use api_token::application::{ApiTokenError, ApiTokenResult, ApiTokenUseCase};
use async_trait::async_trait;
use types::api_token::{
    AdminApiTokenCreate, ApiTokenCreate, ApiTokenCreateResponse, ApiTokenListRequest, ApiTokenListResponse, ApiTokenResponse, ApiTokenSecretResponse,
    ApiTokenUpdate,
};

use crate::llm_proxy::LlmProxyCache;

pub struct ProxyCachedApiTokenUseCase {
    inner: Arc<dyn ApiTokenUseCase>,
    cache: LlmProxyCache,
}

impl ProxyCachedApiTokenUseCase {
    pub fn new(inner: Arc<dyn ApiTokenUseCase>, cache: LlmProxyCache) -> Self {
        Self { inner, cache }
    }

    async fn bump_auth(&self) -> ApiTokenResult<()> {
        self.cache.bump_auth_version().await.map_err(cache_error)
    }
}

#[async_trait]
impl ApiTokenUseCase for ProxyCachedApiTokenUseCase {
    async fn create_token(&self, user_id: &str, input: ApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse> {
        let value = self.inner.create_token(user_id, input).await?;
        self.bump_auth().await?;
        Ok(value)
    }

    async fn update_token(&self, user_id: &str, id: &str, input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse> {
        let value = self.inner.update_token(user_id, id, input).await?;
        self.bump_auth().await?;
        Ok(value)
    }

    async fn delete_token(&self, user_id: &str, id: &str) -> ApiTokenResult<()> {
        self.inner.delete_token(user_id, id).await?;
        self.bump_auth().await
    }

    async fn get_token(&self, user_id: &str, id: &str) -> ApiTokenResult<ApiTokenResponse> {
        self.inner.get_token(user_id, id).await
    }

    async fn token_secret(&self, user_id: &str, id: &str) -> ApiTokenResult<ApiTokenSecretResponse> {
        self.inner.token_secret(user_id, id).await
    }

    async fn list_tokens(&self, user_id: &str, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        self.inner.list_tokens(user_id, request).await
    }

    async fn create_admin_token(&self, actor_id: &str, input: AdminApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse> {
        let value = self.inner.create_admin_token(actor_id, input).await?;
        self.bump_auth().await?;
        Ok(value)
    }

    async fn update_admin_token(&self, id: &str, input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse> {
        let value = self.inner.update_admin_token(id, input).await?;
        self.bump_auth().await?;
        Ok(value)
    }

    async fn delete_admin_token(&self, id: &str) -> ApiTokenResult<()> {
        self.inner.delete_admin_token(id).await?;
        self.bump_auth().await
    }

    async fn get_admin_token(&self, id: &str) -> ApiTokenResult<ApiTokenResponse> {
        self.inner.get_admin_token(id).await
    }

    async fn admin_token_secret(&self, id: &str) -> ApiTokenResult<ApiTokenSecretResponse> {
        self.inner.admin_token_secret(id).await
    }

    async fn list_admin_tokens(&self, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        self.inner.list_admin_tokens(request).await
    }

    async fn cleanup_expired_tokens(&self) -> ApiTokenResult<u64> {
        let deleted = self.inner.cleanup_expired_tokens().await?;
        if deleted > 0 {
            self.bump_auth().await?;
        }
        Ok(deleted)
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> ApiTokenError {
    ApiTokenError::Infrastructure(error.to_string())
}
