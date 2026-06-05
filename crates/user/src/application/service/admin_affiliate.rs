use async_trait::async_trait;
use types::{
    pagination::{Page, PageSliceRequest},
    user::{
        AdminAffiliateCommissionItem, AdminAffiliateCommissionQuery, AdminAffiliateOverviewResponse, AdminAffiliateRelationChangeItem,
        AdminAffiliateRelationChangeQuery, AdminAffiliateRelationItem, AdminAffiliateRelationQuery, AdminAffiliateRelationUpdateRequest,
        AdminAffiliateReportQuery, AdminAffiliateReportResponse, AffiliateRelationChangeRecord,
    },
};

use crate::application::{AdminAffiliateRepository, AdminAffiliateUseCase, AffiliateRelationUpdateRecord, AppError, AppResult, UserRepository};

use super::{UserService, validation::validate_page_slice};

#[async_trait]
impl<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y> AdminAffiliateUseCase for UserService<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y>
where
    R: UserRepository + AdminAffiliateRepository,
    H: Send + Sync + 'static,
    S: Send + Sync + 'static,
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
    async fn admin_affiliate_overview(&self) -> AppResult<AdminAffiliateOverviewResponse> {
        self.repository.admin_affiliate_overview().await
    }

    async fn list_admin_affiliate_relations(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationQuery,
    ) -> AppResult<Page<AdminAffiliateRelationItem>> {
        validate_page_slice(request)?;
        self.repository.page_admin_affiliate_relations(request, query).await
    }

    async fn update_admin_affiliate_relation(
        &self,
        user_id: &str,
        input: AdminAffiliateRelationUpdateRequest,
        operator_user_id: Option<String>,
    ) -> AppResult<AffiliateRelationChangeRecord> {
        let record = relation_update_record(input, operator_user_id)?;
        self.repository.update_affiliate_relation(user_id, record).await
    }

    async fn list_admin_affiliate_relation_changes(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateRelationChangeQuery,
    ) -> AppResult<Page<AdminAffiliateRelationChangeItem>> {
        validate_page_slice(request)?;
        self.repository.page_admin_affiliate_relation_changes(request, query).await
    }

    async fn list_admin_affiliate_commissions(
        &self,
        request: PageSliceRequest,
        query: AdminAffiliateCommissionQuery,
    ) -> AppResult<Page<AdminAffiliateCommissionItem>> {
        validate_page_slice(request)?;
        self.repository.page_admin_affiliate_commissions(request, query).await
    }

    async fn admin_affiliate_report(&self, query: AdminAffiliateReportQuery) -> AppResult<AdminAffiliateReportResponse> {
        validate_report_page(&query)?;
        self.repository.admin_affiliate_report(query).await
    }

    async fn export_admin_affiliate_commissions(&self, query: AdminAffiliateCommissionQuery) -> AppResult<Vec<AdminAffiliateCommissionItem>> {
        self.repository.export_admin_affiliate_commissions(query).await
    }

    async fn export_admin_affiliate_daily_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<types::user::AdminAffiliateDailyReportItem>> {
        self.repository.export_admin_affiliate_daily_report(query).await
    }

    async fn export_admin_affiliate_referrer_report(&self, query: AdminAffiliateReportQuery) -> AppResult<Vec<types::user::AdminAffiliateReferrerReportItem>> {
        self.repository.export_admin_affiliate_referrer_report(query).await
    }
}

fn relation_update_record(input: AdminAffiliateRelationUpdateRequest, operator_user_id: Option<String>) -> AppResult<AffiliateRelationUpdateRecord> {
    let reason = required_text(input.reason, "reason")?;
    let referrer_aff_code = input.referrer_aff_code.and_then(trim_nonempty);
    if !input.clear_referrer && referrer_aff_code.is_none() {
        return Err(AppError::InvalidInput("referrer_aff_code is required".into()));
    }
    Ok(AffiliateRelationUpdateRecord {
        referrer_aff_code,
        clear_referrer: input.clear_referrer,
        reason,
        operator_user_id,
    })
}

fn validate_report_page(query: &AdminAffiliateReportQuery) -> AppResult<()> {
    validate_page_slice(PageSliceRequest {
        offset: query.page.saturating_sub(1).saturating_mul(query.page_size),
        limit: query.page_size,
        page: query.page,
        page_size: query.page_size,
    })
}

fn required_text(value: String, field: &str) -> AppResult<String> {
    trim_nonempty(value).ok_or_else(|| AppError::InvalidInput(format!("{field} is required")))
}

fn trim_nonempty(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return None;
    }
    Some(value)
}
