use async_trait::async_trait;
use types::{
    pagination::{Page, PageSliceRequest},
    user::{
        AdminAffiliateCommissionItem, AdminAffiliateCommissionQuery, AdminAffiliateDailyReportItem, AdminAffiliateOverviewResponse,
        AdminAffiliateReferrerReportItem, AdminAffiliateRelationChangeItem, AdminAffiliateRelationChangeQuery, AdminAffiliateRelationItem,
        AdminAffiliateRelationQuery, AdminAffiliateRelationUpdateRequest, AdminAffiliateReportQuery, AdminAffiliateReportResponse,
        AffiliateRelationChangeRecord,
    },
};

use super::AppResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AffiliateRelationUpdateRecord {
    pub referrer_aff_code: Option<String>,
    pub clear_referrer: bool,
    pub reason: String,
    pub operator_user_id: Option<String>,
}

#[async_trait]
pub trait AdminAffiliateRepository: Send + Sync + 'static {
    async fn admin_affiliate_overview(&self) -> AppResult<AdminAffiliateOverviewResponse>;
    async fn page_admin_affiliate_relations(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationQuery,
    ) -> AppResult<Page<AdminAffiliateRelationItem>>;
    async fn update_affiliate_relation(&self, user_id: &str, input: AffiliateRelationUpdateRecord) -> AppResult<AffiliateRelationChangeRecord>;
    async fn page_admin_affiliate_relation_changes(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationChangeQuery,
    ) -> AppResult<Page<AdminAffiliateRelationChangeItem>>;
    async fn page_admin_affiliate_commissions(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateCommissionQuery,
    ) -> AppResult<Page<AdminAffiliateCommissionItem>>;
    async fn admin_affiliate_report(&self, query: AdminAffiliateReportQuery) -> AppResult<AdminAffiliateReportResponse>;
    async fn export_admin_affiliate_commissions(&self, query: AdminAffiliateCommissionQuery) -> AppResult<Vec<AdminAffiliateCommissionItem>>;
    async fn export_admin_affiliate_daily_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateDailyReportItem>>;
    async fn export_admin_affiliate_referrer_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateReferrerReportItem>>;
}

#[async_trait]
pub trait AdminAffiliateUseCase: Send + Sync + 'static {
    async fn admin_affiliate_overview(&self) -> AppResult<AdminAffiliateOverviewResponse>;
    async fn list_admin_affiliate_relations(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationQuery,
    ) -> AppResult<Page<AdminAffiliateRelationItem>>;
    async fn update_admin_affiliate_relation(
        &self,
        user_id: &str,
        input: AdminAffiliateRelationUpdateRequest,
        operator_user_id: Option<String>,
    ) -> AppResult<AffiliateRelationChangeRecord>;
    async fn list_admin_affiliate_relation_changes(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationChangeQuery,
    ) -> AppResult<Page<AdminAffiliateRelationChangeItem>>;
    async fn list_admin_affiliate_commissions(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateCommissionQuery,
    ) -> AppResult<Page<AdminAffiliateCommissionItem>>;
    async fn admin_affiliate_report(&self, query: AdminAffiliateReportQuery) -> AppResult<AdminAffiliateReportResponse>;
    async fn export_admin_affiliate_commissions(&self, query: AdminAffiliateCommissionQuery) -> AppResult<Vec<AdminAffiliateCommissionItem>>;
    async fn export_admin_affiliate_daily_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateDailyReportItem>>;
    async fn export_admin_affiliate_referrer_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateReferrerReportItem>>;
}
