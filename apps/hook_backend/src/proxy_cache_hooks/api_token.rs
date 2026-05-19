use api_token::application::{ApiTokenCreateRecord, ApiTokenError, ApiTokenRepository, ApiTokenResult, ApiTokenUpdateRecord};
use async_trait::async_trait;
use types::api_token::{ApiToken, ApiTokenListRequest, ApiTokenListResponse};

use super::cache::ProxyCacheInvalidator;

#[derive(Clone)]
pub struct CachedApiTokenRepository<R, C> {
    inner: R,
    cache: C,
}

impl<R, C> CachedApiTokenRepository<R, C> {
    pub const fn new(inner: R, cache: C) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl<R, C> ApiTokenRepository for CachedApiTokenRepository<R, C>
where
    R: ApiTokenRepository,
    C: ProxyCacheInvalidator,
{
    async fn create_token(&self, input: ApiTokenCreateRecord) -> ApiTokenResult<ApiToken> {
        let token = self.inner.create_token(input).await?;
        self.bump_auth().await?;
        Ok(token)
    }

    async fn update_token(&self, user_id: &str, id: &str, input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        let token = self.inner.update_token(user_id, id, input).await?;
        self.bump_auth().await?;
        Ok(token)
    }

    async fn update_any_token(&self, id: &str, input: ApiTokenUpdateRecord) -> ApiTokenResult<ApiToken> {
        let token = self.inner.update_any_token(id, input).await?;
        self.bump_auth().await?;
        Ok(token)
    }

    async fn delete_token(&self, user_id: &str, id: &str) -> ApiTokenResult<()> {
        self.inner.delete_token(user_id, id).await?;
        self.bump_auth().await
    }

    async fn delete_any_token(&self, id: &str) -> ApiTokenResult<()> {
        self.inner.delete_any_token(id).await?;
        self.bump_auth().await
    }

    async fn find_user_token(&self, user_id: &str, id: &str) -> ApiTokenResult<Option<ApiToken>> {
        self.inner.find_user_token(user_id, id).await
    }

    async fn find_token(&self, id: &str) -> ApiTokenResult<Option<ApiToken>> {
        self.inner.find_token(id).await
    }

    async fn find_by_hash(&self, token_hash: &str) -> ApiTokenResult<Option<ApiToken>> {
        self.inner.find_by_hash(token_hash).await
    }

    async fn list_user_tokens(&self, user_id: &str, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        self.inner.list_user_tokens(user_id, request).await
    }

    async fn list_admin_tokens(&self, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        self.inner.list_admin_tokens(request).await
    }

    async fn delete_expired_tokens(&self) -> ApiTokenResult<u64> {
        let deleted = self.inner.delete_expired_tokens().await?;
        if deleted > 0 {
            self.bump_auth().await?;
        }
        Ok(deleted)
    }
}

impl<R, C> CachedApiTokenRepository<R, C>
where
    C: ProxyCacheInvalidator,
{
    async fn bump_auth(&self) -> ApiTokenResult<()> {
        self.cache.bump_auth().await.map_err(cache_error)
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> ApiTokenError {
    ApiTokenError::Infrastructure(error.to_string())
}
