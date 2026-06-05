use async_trait::async_trait;
use types::{
    pagination::{Page, PageSliceRequest},
    user::{AffiliateCommissionItem, AffiliateCommissionQuery, AffiliateReferralItem, AffiliateReferralQuery, AffiliateSummaryResponse, UserId},
};

use super::AppResult;

#[async_trait]
pub trait AffiliateRepository: Send + Sync + 'static {
    async fn affiliate_summary(&self, user_id: &str, affiliate_code: &str) -> AppResult<AffiliateSummaryResponse>;
    async fn page_affiliate_referrals(&self, user_id: &str, request: PageSliceRequest, query: AffiliateReferralQuery)
    -> AppResult<Page<AffiliateReferralItem>>;
    async fn page_affiliate_commissions(
        &self,
        user_id: &str,
        request: PageSliceRequest,
        query: AffiliateCommissionQuery,
    ) -> AppResult<Page<AffiliateCommissionItem>>;
    async fn export_affiliate_commissions(&self, user_id: &str, query: AffiliateCommissionQuery) -> AppResult<Vec<AffiliateCommissionItem>>;
}

#[async_trait]
pub trait AffiliateUseCase: Send + Sync + 'static {
    async fn affiliate_summary(&self, id: UserId) -> AppResult<AffiliateSummaryResponse>;
    async fn list_affiliate_referrals(&self, id: UserId, request: PageSliceRequest, query: AffiliateReferralQuery) -> AppResult<Page<AffiliateReferralItem>>;
    async fn list_affiliate_commissions(
        &self,
        id: UserId,
        request: PageSliceRequest,
        query: AffiliateCommissionQuery,
    ) -> AppResult<Page<AffiliateCommissionItem>>;
    async fn export_affiliate_commissions(&self, id: UserId, query: AffiliateCommissionQuery) -> AppResult<Vec<AffiliateCommissionItem>>;
}
