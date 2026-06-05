mod filter;
mod query;
mod rows;

use types::{
    pagination::{Page, PageSliceRequest},
    user::{AffiliateCommissionItem, AffiliateCommissionQuery, AffiliateReferralItem, AffiliateReferralQuery, AffiliateSummaryResponse},
};

use crate::StorageResult;

impl super::UserStore {
    pub async fn affiliate_summary(&self, user_id: &str, affiliate_code: &str) -> StorageResult<AffiliateSummaryResponse> {
        query::summary(self, user_id, affiliate_code).await
    }

    pub async fn page_affiliate_referrals(
        &self,
        user_id: &str,
        request: PageSliceRequest,
        query: AffiliateReferralQuery,
    ) -> StorageResult<Page<AffiliateReferralItem>> {
        self::query::referrals(self, user_id, request, query).await
    }

    pub async fn page_affiliate_commissions(
        &self,
        user_id: &str,
        request: PageSliceRequest,
        query: AffiliateCommissionQuery,
    ) -> StorageResult<Page<AffiliateCommissionItem>> {
        self::query::commissions(self, user_id, request, query).await
    }

    pub async fn export_affiliate_commissions(&self, user_id: &str, query: AffiliateCommissionQuery) -> StorageResult<Vec<AffiliateCommissionItem>> {
        self::query::export_commissions(self, user_id, query).await
    }
}
