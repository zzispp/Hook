use async_trait::async_trait;
use types::{
    pagination::{Page, PageSliceRequest},
    user::{AffiliateCommissionItem, AffiliateCommissionQuery, AffiliateReferralItem, AffiliateReferralQuery, AffiliateSummaryResponse},
};

use crate::{
    application::{AffiliateRepository, AppResult},
    infra::user_repository::{StorageUserRepository, storage_error},
};

#[async_trait]
impl AffiliateRepository for StorageUserRepository {
    async fn affiliate_summary(&self, user_id: &str, affiliate_code: &str) -> AppResult<AffiliateSummaryResponse> {
        self.store.affiliate_summary(user_id, affiliate_code).await.map_err(storage_error)
    }

    async fn page_affiliate_referrals(
        &self,
        user_id: &str,
        request: PageSliceRequest,
        query: AffiliateReferralQuery,
    ) -> AppResult<Page<AffiliateReferralItem>> {
        self.store.page_affiliate_referrals(user_id, request, query).await.map_err(storage_error)
    }

    async fn page_affiliate_commissions(
        &self,
        user_id: &str,
        request: PageSliceRequest,
        query: AffiliateCommissionQuery,
    ) -> AppResult<Page<AffiliateCommissionItem>> {
        self.store.page_affiliate_commissions(user_id, request, query).await.map_err(storage_error)
    }

    async fn export_affiliate_commissions(&self, user_id: &str, query: AffiliateCommissionQuery) -> AppResult<Vec<AffiliateCommissionItem>> {
        self.store.export_affiliate_commissions(user_id, query).await.map_err(storage_error)
    }
}
