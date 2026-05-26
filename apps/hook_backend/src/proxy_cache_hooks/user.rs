use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{User, UserId, UserListFilters},
    user_group::{UserGroupListRequest, UserGroupPageResponse, UserGroupResponse},
};
use user::application::{
    AppError, AppResult, PasswordResetRecord, PasswordResetRepository, ReplaceUserRecord, UserAuthRecord, UserGroupCreateRecord, UserGroupRepository,
    UserGroupUpdateRecord, UserRepository,
};

use super::cache::{ProxyCacheInvalidator, combine_cache_results};

#[derive(Clone)]
pub struct CachedUserRepository<R, C> {
    inner: R,
    cache: C,
}

impl<R, C> CachedUserRepository<R, C> {
    pub const fn new(inner: R, cache: C) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl<R, C> UserRepository for CachedUserRepository<R, C>
where
    R: UserRepository,
    C: ProxyCacheInvalidator,
{
    async fn create(&self, user: ReplaceUserRecord) -> AppResult<User> {
        let user = self.inner.create(user).await?;
        self.refresh_scheduling().await?;
        Ok(user)
    }

    async fn replace(&self, id: UserId, user: ReplaceUserRecord) -> AppResult<User> {
        let user = self.inner.replace(id, user).await?;
        self.refresh_scheduling().await?;
        Ok(user)
    }

    async fn delete(&self, id: UserId) -> AppResult<()> {
        self.inner.delete(id).await?;
        let auth_result = self.cache.bump_auth().await;
        let scheduling_result = self.cache.refresh_scheduling().await;
        combine_cache_results(auth_result, scheduling_result).map_err(cache_error)
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        self.inner.find_by_id(id).await
    }

    async fn find_auth_by_id(&self, id: UserId) -> AppResult<Option<UserAuthRecord>> {
        self.inner.find_auth_by_id(id).await
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        self.inner.find_by_email(email).await
    }

    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>> {
        self.inner.find_auth_by_username(username).await
    }

    async fn find_auth_by_email(&self, email: &str) -> AppResult<Option<UserAuthRecord>> {
        self.inner.find_auth_by_email(email).await
    }

    async fn record_login(&self, id: UserId) -> AppResult<()> {
        self.inner.record_login(id).await
    }

    async fn list(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        self.inner.list(page, filters).await
    }

    async fn list_slice(&self, request: PageSliceRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        self.inner.list_slice(request, filters).await
    }
}

#[async_trait]
impl<R, C> PasswordResetRepository for CachedUserRepository<R, C>
where
    R: PasswordResetRepository,
    C: ProxyCacheInvalidator,
{
    async fn create_password_reset_token(&self, record: PasswordResetRecord) -> AppResult<()> {
        self.inner.create_password_reset_token(record).await
    }

    async fn consume_password_reset_token(&self, token_hash: &str, password_hash: &str, now: time::OffsetDateTime) -> AppResult<Option<User>> {
        self.inner.consume_password_reset_token(token_hash, password_hash, now).await
    }
}

#[async_trait]
impl<R, C> UserGroupRepository for CachedUserRepository<R, C>
where
    R: UserGroupRepository,
    C: ProxyCacheInvalidator,
{
    async fn create_group(&self, input: UserGroupCreateRecord) -> AppResult<UserGroupResponse> {
        let group = self.inner.create_group(input).await?;
        self.refresh_scheduling().await?;
        Ok(group)
    }

    async fn update_group(&self, code: &str, input: UserGroupUpdateRecord) -> AppResult<UserGroupResponse> {
        let group = self.inner.update_group(code, input).await?;
        self.refresh_scheduling().await?;
        Ok(group)
    }

    async fn delete_group(&self, code: &str) -> AppResult<()> {
        self.inner.delete_group(code).await?;
        self.refresh_scheduling().await
    }

    async fn find_group(&self, code: &str) -> AppResult<Option<UserGroupResponse>> {
        self.inner.find_group(code).await
    }

    async fn list_groups(&self, request: UserGroupListRequest) -> AppResult<UserGroupPageResponse> {
        self.inner.list_groups(request).await
    }

    async fn group_has_users(&self, code: &str) -> AppResult<bool> {
        self.inner.group_has_users(code).await
    }

    async fn list_group_users(&self, request: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        self.inner.list_group_users(request, filters).await
    }
}

impl<R, C> CachedUserRepository<R, C>
where
    C: ProxyCacheInvalidator,
{
    async fn refresh_scheduling(&self) -> AppResult<()> {
        self.cache.refresh_scheduling().await.map_err(cache_error)
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> AppError {
    AppError::Infrastructure(error.to_string())
}

#[cfg(test)]
#[path = "user_tests.rs"]
mod user_tests;
