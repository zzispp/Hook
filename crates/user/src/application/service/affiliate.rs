use async_trait::async_trait;
use types::{
    pagination::{Page, PageSliceRequest},
    user::{AffiliateCommissionItem, AffiliateCommissionQuery, AffiliateReferralItem, AffiliateReferralQuery, AffiliateSummaryResponse, UserId},
};

use crate::application::{AffiliateRepository, AffiliateUseCase, AppError, AppResult, SystemUserProvider, UserRepository};

use super::{UserService, system_user::system_user_by_id, validation::validate_page_slice};

#[async_trait]
impl<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y> AffiliateUseCase for UserService<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y>
where
    R: UserRepository + AffiliateRepository,
    H: Send + Sync + 'static,
    S: SystemUserProvider,
    P: Send + Sync + 'static,
    G: Send + Sync + 'static,
    W: Send + Sync + 'static,
    C: Send + Sync + 'static,
    M: Send + Sync + 'static,
    E: Send + Sync + 'static,
    N: Send + Sync + 'static,
    K: Send + Sync + 'static,
    A: Send + Sync + 'static,
    O: Send + Sync + 'static,
    T: Send + Sync + 'static,
    Y: Send + Sync + 'static,
{
    async fn affiliate_summary(&self, id: UserId) -> AppResult<AffiliateSummaryResponse> {
        let user = authenticated_user(&self.repository, &self.system_users, id).await?;
        self.repository.affiliate_summary(&user.id.0, &user.affiliate_code).await
    }

    async fn list_affiliate_referrals(&self, id: UserId, request: PageSliceRequest, query: AffiliateReferralQuery) -> AppResult<Page<AffiliateReferralItem>> {
        validate_page_slice(request)?;
        let user = authenticated_user(&self.repository, &self.system_users, id).await?;
        self.repository.page_affiliate_referrals(&user.id.0, request, query).await
    }

    async fn list_affiliate_commissions(
        &self,
        id: UserId,
        request: PageSliceRequest,
        query: AffiliateCommissionQuery,
    ) -> AppResult<Page<AffiliateCommissionItem>> {
        validate_page_slice(request)?;
        let user = authenticated_user(&self.repository, &self.system_users, id).await?;
        self.repository.page_affiliate_commissions(&user.id.0, request, query).await
    }

    async fn export_affiliate_commissions(&self, id: UserId, query: AffiliateCommissionQuery) -> AppResult<Vec<AffiliateCommissionItem>> {
        let user = authenticated_user(&self.repository, &self.system_users, id).await?;
        self.repository.export_affiliate_commissions(&user.id.0, query).await
    }
}

async fn authenticated_user<R, S>(repository: &R, system_users: &S, id: UserId) -> AppResult<types::user::User>
where
    R: UserRepository,
    S: SystemUserProvider,
{
    if let Some(system_user) = system_user_by_id(system_users, &id) {
        return Ok(system_user.user);
    }
    repository.find_by_id(id).await?.ok_or(AppError::Unauthorized)
}
