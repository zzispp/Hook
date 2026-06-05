use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{
        AdminAffiliateCommissionItem, AdminAffiliateCommissionQuery, AdminAffiliateDailyReportItem, AdminAffiliateOverviewResponse,
        AdminAffiliateReferrerReportItem, AdminAffiliateRelationChangeItem, AdminAffiliateRelationChangeQuery, AdminAffiliateRelationItem,
        AdminAffiliateRelationQuery, AdminAffiliateReportQuery, AdminAffiliateReportResponse, AffiliateCommissionItem, AffiliateCommissionQuery,
        AffiliateReferralItem, AffiliateReferralQuery, AffiliateRelationChangeRecord, AffiliateSummaryResponse, IdentityProvider, User, UserId, UserIdentity,
        UserIdentityInput, UserListFilters,
    },
    user_group::{UserGroupListRequest, UserGroupPageResponse, UserGroupResponse},
};
use user::application::{
    AdminAffiliateRepository, AffiliateRelationUpdateRecord, AffiliateRepository, AppError, AppResult, PasswordResetRecord, PasswordResetRepository,
    ReplaceUserRecord, UserAuthRecord, UserGroupCreateRecord, UserGroupRepository, UserGroupUpdateRecord, UserRepository,
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

    async fn find_by_affiliate_code(&self, affiliate_code: &str) -> AppResult<Option<User>> {
        self.inner.find_by_affiliate_code(affiliate_code).await
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

    async fn create_identity(&self, input: UserIdentityInput) -> AppResult<UserIdentity> {
        let identity = self.inner.create_identity(input).await?;
        self.refresh_scheduling().await?;
        Ok(identity)
    }

    async fn find_identity(&self, provider: IdentityProvider, subject: &str) -> AppResult<Option<UserIdentity>> {
        self.inner.find_identity(provider, subject).await
    }

    async fn list_identities_by_user_id(&self, user_id: &str) -> AppResult<Vec<UserIdentity>> {
        self.inner.list_identities_by_user_id(user_id).await
    }

    async fn list_identities_by_user_ids(&self, user_ids: &[String]) -> AppResult<std::collections::BTreeMap<String, Vec<UserIdentity>>> {
        self.inner.list_identities_by_user_ids(user_ids).await
    }

    async fn touch_identity_login(&self, identity_id: &str) -> AppResult<()> {
        self.inner.touch_identity_login(identity_id).await
    }

    async fn delete_identity(&self, identity_id: &str) -> AppResult<()> {
        self.inner.delete_identity(identity_id).await?;
        self.refresh_scheduling().await
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
impl<R, C> AffiliateRepository for CachedUserRepository<R, C>
where
    R: AffiliateRepository,
    C: ProxyCacheInvalidator,
{
    async fn affiliate_summary(&self, user_id: &str, affiliate_code: &str) -> AppResult<AffiliateSummaryResponse> {
        self.inner.affiliate_summary(user_id, affiliate_code).await
    }

    async fn page_affiliate_referrals(
        &self,
        user_id: &str,
        request: PageSliceRequest,
        query: AffiliateReferralQuery,
    ) -> AppResult<Page<AffiliateReferralItem>> {
        self.inner.page_affiliate_referrals(user_id, request, query).await
    }

    async fn page_affiliate_commissions(
        &self,
        user_id: &str,
        request: PageSliceRequest,
        query: AffiliateCommissionQuery,
    ) -> AppResult<Page<AffiliateCommissionItem>> {
        self.inner.page_affiliate_commissions(user_id, request, query).await
    }

    async fn export_affiliate_commissions(&self, user_id: &str, query: AffiliateCommissionQuery) -> AppResult<Vec<AffiliateCommissionItem>> {
        self.inner.export_affiliate_commissions(user_id, query).await
    }
}

#[async_trait]
impl<R, C> AdminAffiliateRepository for CachedUserRepository<R, C>
where
    R: AdminAffiliateRepository,
    C: ProxyCacheInvalidator,
{
    async fn admin_affiliate_overview(&self) -> AppResult<AdminAffiliateOverviewResponse> {
        self.inner.admin_affiliate_overview().await
    }

    async fn page_admin_affiliate_relations(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationQuery,
    ) -> AppResult<Page<AdminAffiliateRelationItem>> {
        self.inner.page_admin_affiliate_relations(request, query).await
    }

    async fn update_affiliate_relation(&self, user_id: &str, input: AffiliateRelationUpdateRecord) -> AppResult<AffiliateRelationChangeRecord> {
        let record = self.inner.update_affiliate_relation(user_id, input).await?;
        self.refresh_scheduling().await?;
        Ok(record)
    }

    async fn page_admin_affiliate_relation_changes(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationChangeQuery,
    ) -> AppResult<Page<AdminAffiliateRelationChangeItem>> {
        self.inner.page_admin_affiliate_relation_changes(request, query).await
    }

    async fn page_admin_affiliate_commissions(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateCommissionQuery,
    ) -> AppResult<Page<AdminAffiliateCommissionItem>> {
        self.inner.page_admin_affiliate_commissions(request, query).await
    }

    async fn admin_affiliate_report(&self, query: AdminAffiliateReportQuery) -> AppResult<AdminAffiliateReportResponse> {
        self.inner.admin_affiliate_report(query).await
    }

    async fn export_admin_affiliate_commissions(&self, query: AdminAffiliateCommissionQuery) -> AppResult<Vec<AdminAffiliateCommissionItem>> {
        self.inner.export_admin_affiliate_commissions(query).await
    }

    async fn export_admin_affiliate_daily_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateDailyReportItem>> {
        self.inner.export_admin_affiliate_daily_report(query).await
    }

    async fn export_admin_affiliate_referrer_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateReferrerReportItem>> {
        self.inner.export_admin_affiliate_referrer_report(query).await
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
