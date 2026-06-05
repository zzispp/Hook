mod filter;
mod mutation;
mod query;
mod relation_change_query;
mod rows;

pub use mutation::AffiliateRelationUpdateInput;

use types::{
    pagination::{Page, PageSliceRequest},
    user::{
        AdminAffiliateCommissionItem, AdminAffiliateCommissionQuery, AdminAffiliateDailyReportItem, AdminAffiliateOverviewResponse,
        AdminAffiliateReferrerReportItem, AdminAffiliateRelationChangeItem, AdminAffiliateRelationChangeQuery, AdminAffiliateRelationItem,
        AdminAffiliateRelationQuery, AdminAffiliateReportQuery, AdminAffiliateReportResponse, AffiliateRelationChangeRecord,
    },
};

use crate::StorageResult;

impl super::UserStore {
    pub async fn admin_affiliate_overview(&self) -> StorageResult<AdminAffiliateOverviewResponse> {
        query::overview(self).await
    }

    pub async fn page_admin_affiliate_relations(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationQuery,
    ) -> StorageResult<Page<AdminAffiliateRelationItem>> {
        query::relations(self, request, query).await
    }

    pub async fn update_affiliate_relation(&self, user_id: &str, input: AffiliateRelationUpdateInput) -> StorageResult<AffiliateRelationChangeRecord> {
        mutation::update_relation(self, user_id, input).await
    }

    pub async fn page_admin_affiliate_relation_changes(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationChangeQuery,
    ) -> StorageResult<Page<AdminAffiliateRelationChangeItem>> {
        relation_change_query::relation_changes(self, request, query).await
    }

    pub async fn page_admin_affiliate_commissions(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateCommissionQuery,
    ) -> StorageResult<Page<AdminAffiliateCommissionItem>> {
        query::commissions(self, request, query).await
    }

    pub async fn admin_affiliate_report(&self, query: AdminAffiliateReportQuery) -> StorageResult<AdminAffiliateReportResponse> {
        query::report(self, query).await
    }

    pub async fn export_admin_affiliate_commissions(&self, query: AdminAffiliateCommissionQuery) -> StorageResult<Vec<AdminAffiliateCommissionItem>> {
        query::export_commissions(self, query).await
    }

    pub async fn export_admin_affiliate_daily_report(&self, query: AdminAffiliateReportQuery) -> StorageResult<Vec<AdminAffiliateDailyReportItem>> {
        query::export_daily_report(self, query).await
    }

    pub async fn export_admin_affiliate_referrer_report(&self, query: AdminAffiliateReportQuery) -> StorageResult<Vec<AdminAffiliateReferrerReportItem>> {
        query::export_referrer_report(self, query).await
    }
}
