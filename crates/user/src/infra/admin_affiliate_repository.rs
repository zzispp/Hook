use async_trait::async_trait;
use storage::user::AffiliateRelationUpdateInput;
use types::{
    pagination::{Page, PageSliceRequest},
    user::{
        AdminAffiliateCommissionItem, AdminAffiliateCommissionQuery, AdminAffiliateDailyReportItem, AdminAffiliateOverviewResponse,
        AdminAffiliateReferrerReportItem, AdminAffiliateRelationChangeItem, AdminAffiliateRelationChangeQuery, AdminAffiliateRelationItem,
        AdminAffiliateRelationQuery, AdminAffiliateReportQuery, AdminAffiliateReportResponse, AffiliateRelationChangeRecord,
    },
};

use crate::{
    application::{AdminAffiliateRepository, AffiliateRelationUpdateRecord, AppResult},
    infra::user_repository::{StorageUserRepository, storage_error},
};

#[async_trait]
impl AdminAffiliateRepository for StorageUserRepository {
    async fn admin_affiliate_overview(&self) -> AppResult<AdminAffiliateOverviewResponse> {
        self.store.admin_affiliate_overview().await.map_err(storage_error)
    }

    async fn page_admin_affiliate_relations(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationQuery,
    ) -> AppResult<Page<AdminAffiliateRelationItem>> {
        self.store.page_admin_affiliate_relations(request, query).await.map_err(storage_error)
    }

    async fn update_affiliate_relation(&self, user_id: &str, input: AffiliateRelationUpdateRecord) -> AppResult<AffiliateRelationChangeRecord> {
        self.store
            .update_affiliate_relation(user_id, storage_relation_update(input))
            .await
            .map_err(storage_error)
    }

    async fn page_admin_affiliate_relation_changes(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationChangeQuery,
    ) -> AppResult<Page<AdminAffiliateRelationChangeItem>> {
        self.store.page_admin_affiliate_relation_changes(request, query).await.map_err(storage_error)
    }

    async fn page_admin_affiliate_commissions(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateCommissionQuery,
    ) -> AppResult<Page<AdminAffiliateCommissionItem>> {
        self.store.page_admin_affiliate_commissions(request, query).await.map_err(storage_error)
    }

    async fn admin_affiliate_report(&self, query: AdminAffiliateReportQuery) -> AppResult<AdminAffiliateReportResponse> {
        self.store.admin_affiliate_report(query).await.map_err(storage_error)
    }

    async fn export_admin_affiliate_commissions(&self, query: AdminAffiliateCommissionQuery) -> AppResult<Vec<AdminAffiliateCommissionItem>> {
        self.store.export_admin_affiliate_commissions(query).await.map_err(storage_error)
    }

    async fn export_admin_affiliate_daily_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateDailyReportItem>> {
        self.store.export_admin_affiliate_daily_report(query).await.map_err(storage_error)
    }

    async fn export_admin_affiliate_referrer_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<AdminAffiliateReferrerReportItem>> {
        self.store.export_admin_affiliate_referrer_report(query).await.map_err(storage_error)
    }
}

fn storage_relation_update(input: AffiliateRelationUpdateRecord) -> AffiliateRelationUpdateInput {
    AffiliateRelationUpdateInput {
        referrer_aff_code: input.referrer_aff_code,
        clear_referrer: input.clear_referrer,
        reason: input.reason,
        operator_user_id: input.operator_user_id,
    }
}
