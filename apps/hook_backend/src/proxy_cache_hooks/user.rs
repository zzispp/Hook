use std::sync::Arc;

use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest},
    user::{Credentials, NewUser, ReplaceUser, User, UserId, UserListFilters, UserWalletSummaryResponse},
};
use user::application::{AppError, AppResult, UserUseCase};

use crate::llm_proxy::LlmProxyCache;

pub struct ProxyCachedUserUseCase {
    inner: Arc<dyn UserUseCase>,
    cache: LlmProxyCache,
}

impl ProxyCachedUserUseCase {
    pub fn new(inner: Arc<dyn UserUseCase>, cache: LlmProxyCache) -> Self {
        Self { inner, cache }
    }

    async fn refresh_proxy_snapshot(&self) -> AppResult<()> {
        self.cache.refresh_scheduling_snapshot().await.map(|_| ()).map_err(cache_error)
    }

    async fn bump_proxy_auth(&self) -> AppResult<()> {
        self.cache.bump_auth_version().await.map_err(cache_error)
    }
}

#[async_trait]
impl UserUseCase for ProxyCachedUserUseCase {
    async fn sign_up(&self, input: NewUser) -> AppResult<User> {
        self.inner.sign_up(input).await
    }

    async fn sign_in(&self, input: Credentials) -> AppResult<User> {
        self.inner.sign_in(input).await
    }

    async fn authenticated_user(&self, id: UserId) -> AppResult<User> {
        self.inner.authenticated_user(id).await
    }

    async fn create_user(&self, input: NewUser) -> AppResult<User> {
        let user = self.inner.create_user(input).await?;
        self.refresh_proxy_snapshot().await?;
        Ok(user)
    }

    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User> {
        let user = self.inner.replace_user(id, input).await?;
        self.refresh_proxy_snapshot().await?;
        Ok(user)
    }

    async fn delete_user(&self, id: UserId) -> AppResult<()> {
        self.inner.delete_user(id).await?;
        self.bump_proxy_auth().await?;
        self.refresh_proxy_snapshot().await
    }

    async fn list_users(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        self.inner.list_users(page, filters).await
    }

    async fn wallet_summaries(&self, user_ids: &[String]) -> AppResult<std::collections::BTreeMap<String, UserWalletSummaryResponse>> {
        self.inner.wallet_summaries(user_ids).await
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> AppError {
    AppError::Infrastructure(error.to_string())
}
