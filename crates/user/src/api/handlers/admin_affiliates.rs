use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use rbac::api::CurrentUser;
use serde::Deserialize;
use types::{
    pagination::PageSliceRequest,
    user::{
        AdminAffiliateCommissionListResponse, AdminAffiliateCommissionQuery, AdminAffiliateDailyReportItem, AdminAffiliateOverviewResponse,
        AdminAffiliateReferrerReportItem, AdminAffiliateRelationChangeListResponse, AdminAffiliateRelationChangeQuery, AdminAffiliateRelationListResponse,
        AdminAffiliateRelationQuery, AdminAffiliateRelationUpdateRequest, AdminAffiliateReportQuery, AdminAffiliateReportResponse,
        AffiliateRelationChangeRecord,
    },
};

use crate::api::{ApiState, error::ApiError, handlers::shared::*};
use crate::application::AppError;

const EXPORT_TYPE_DAILY: &str = "daily";
const EXPORT_TYPE_REFERRERS: &str = "referrers";
const CSV_CONTENT_DISPOSITION: &str = "attachment; filename=\"affiliate-report.csv\"";

#[derive(Clone, Debug, Deserialize)]
pub struct AffiliateExportQuery {
    #[serde(default)]
    pub export_type: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub referrer_search: Option<String>,
    #[serde(default)]
    pub referred_search: Option<String>,
    #[serde(default)]
    pub recharge_order_id: Option<String>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub min_commission_amount: Option<rust_decimal::Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub max_commission_amount: Option<rust_decimal::Decimal>,
}

pub async fn admin_affiliates_overview(State(state): State<ApiState>) -> ApiResult<ApiJson<AdminAffiliateOverviewResponse>> {
    Ok(ok(state.admin_affiliates.admin_affiliate_overview().await?))
}

pub async fn admin_affiliates_relations(
    State(state): State<ApiState>,
    Query(query): Query<AdminAffiliateRelationQuery>,
) -> ApiResult<ApiJson<AdminAffiliateRelationListResponse>> {
    let request = relation_page_request(&query);
    let page = state.admin_affiliates.list_admin_affiliate_relations(request, query).await?;
    Ok(ok(AdminAffiliateRelationListResponse {
        items: page.items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }))
}

pub async fn update_admin_affiliate_relation(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(user_id): Path<String>,
    Json(payload): Json<AdminAffiliateRelationUpdateRequest>,
) -> ApiResult<ApiJson<AffiliateRelationChangeRecord>> {
    Ok(ok(state
        .admin_affiliates
        .update_admin_affiliate_relation(&user_id, payload, operator_user_id(&current_user))
        .await?))
}

pub async fn admin_affiliates_relation_changes(
    State(state): State<ApiState>,
    Query(query): Query<AdminAffiliateRelationChangeQuery>,
) -> ApiResult<ApiJson<AdminAffiliateRelationChangeListResponse>> {
    let request = relation_change_page_request(&query);
    let page = state.admin_affiliates.list_admin_affiliate_relation_changes(request, query).await?;
    Ok(ok(AdminAffiliateRelationChangeListResponse {
        items: page.items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }))
}

pub async fn admin_affiliates_commissions(
    State(state): State<ApiState>,
    Query(query): Query<AdminAffiliateCommissionQuery>,
) -> ApiResult<ApiJson<AdminAffiliateCommissionListResponse>> {
    let request = commission_page_request(&query);
    let page = state.admin_affiliates.list_admin_affiliate_commissions(request, query).await?;
    Ok(ok(AdminAffiliateCommissionListResponse {
        items: page.items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }))
}

pub async fn admin_affiliates_reports(
    State(state): State<ApiState>,
    Query(query): Query<AdminAffiliateReportQuery>,
) -> ApiResult<ApiJson<AdminAffiliateReportResponse>> {
    Ok(ok(state.admin_affiliates.admin_affiliate_report(query).await?))
}

pub async fn export_admin_affiliates_report(State(state): State<ApiState>, Query(query): Query<AffiliateExportQuery>) -> ApiResult<Response> {
    let csv = match query.export_type.as_deref() {
        Some(EXPORT_TYPE_DAILY) => daily_report_csv(state.admin_affiliates.export_admin_affiliate_daily_report(report_query(&query)).await?),
        Some(EXPORT_TYPE_REFERRERS) => referrer_report_csv(state.admin_affiliates.export_admin_affiliate_referrer_report(report_query(&query)).await?),
        Some(value) => return Err(ApiError(AppError::InvalidInput(format!("unsupported export_type: {value}")))),
        None => commission_csv(state.admin_affiliates.export_admin_affiliate_commissions(commission_query(query)).await?),
    };
    Ok(csv_response(csv))
}

fn report_query(query: &AffiliateExportQuery) -> AdminAffiliateReportQuery {
    AdminAffiliateReportQuery {
        page: constants::pagination::MIN_PAGE_NUMBER,
        page_size: constants::pagination::MIN_PAGE_SIZE,
        start_date: query.start_date.clone().or_else(|| query.start_at.clone()),
        end_date: query.end_date.clone().or_else(|| query.end_at.clone()),
        referrer_search: query.referrer_search.clone(),
        referred_search: query.referred_search.clone(),
        export_type: query.export_type.clone(),
    }
}

fn commission_query(query: AffiliateExportQuery) -> AdminAffiliateCommissionQuery {
    AdminAffiliateCommissionQuery {
        page: constants::pagination::MIN_PAGE_NUMBER,
        page_size: constants::pagination::MIN_PAGE_SIZE,
        referrer_search: query.referrer_search,
        referred_search: query.referred_search,
        recharge_order_id: query.recharge_order_id,
        start_at: query.start_at.or(query.start_date),
        end_at: query.end_at.or(query.end_date),
        min_commission_amount: query.min_commission_amount,
        max_commission_amount: query.max_commission_amount,
    }
}

fn relation_page_request(query: &AdminAffiliateRelationQuery) -> PageSliceRequest {
    page_slice(query.page, query.page_size)
}

fn relation_change_page_request(query: &AdminAffiliateRelationChangeQuery) -> PageSliceRequest {
    page_slice(query.page, query.page_size)
}

fn commission_page_request(query: &AdminAffiliateCommissionQuery) -> PageSliceRequest {
    page_slice(query.page, query.page_size)
}

fn page_slice(page: u64, page_size: u64) -> PageSliceRequest {
    PageSliceRequest {
        offset: page.saturating_sub(1).saturating_mul(page_size),
        limit: page_size,
        page,
        page_size,
    }
}

fn csv_response(csv: String) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, CSV_CONTENT_DISPOSITION),
        ],
        csv,
    )
        .into_response()
}

fn commission_csv(items: Vec<types::user::AdminAffiliateCommissionItem>) -> String {
    let rows = items.into_iter().map(|item| {
        vec![
            item.id,
            item.referrer.id,
            item.referrer.username,
            item.referred.id,
            item.referred.username,
            item.recharge_order_id,
            item.payable_amount.to_string(),
            item.commission_percent.to_string(),
            item.commission_amount.to_string(),
            item.wallet_transaction_id.unwrap_or_default(),
            item.status,
            item.failure_reason.unwrap_or_default(),
            item.created_at,
        ]
    });
    csv_string(
        vec![
            "id",
            "referrer_id",
            "referrer_username",
            "referred_id",
            "referred_username",
            "recharge_order_id",
            "payable_amount",
            "commission_percent",
            "commission_amount",
            "wallet_transaction_id",
            "status",
            "failure_reason",
            "created_at",
        ],
        rows,
    )
}

fn daily_report_csv(items: Vec<AdminAffiliateDailyReportItem>) -> String {
    let rows = items.into_iter().map(|item| {
        vec![
            item.date,
            item.commission_order_count.to_string(),
            item.referred_payer_count.to_string(),
            item.payable_amount.to_string(),
            item.commission_amount.to_string(),
        ]
    });
    csv_string(
        vec!["date", "commission_order_count", "referred_payer_count", "payable_amount", "commission_amount"],
        rows,
    )
}

fn referrer_report_csv(items: Vec<AdminAffiliateReferrerReportItem>) -> String {
    let rows = items.into_iter().map(|item| {
        vec![
            item.referrer.id,
            item.referrer.username,
            item.referrer.email,
            item.referrer.affiliate_code,
            item.referred_user_count.to_string(),
            item.commission_order_count.to_string(),
            item.payable_amount.to_string(),
            item.commission_amount.to_string(),
        ]
    });
    csv_string(
        vec![
            "referrer_id",
            "referrer_username",
            "referrer_email",
            "referrer_affiliate_code",
            "referred_user_count",
            "commission_order_count",
            "payable_amount",
            "commission_amount",
        ],
        rows,
    )
}

fn csv_string<I>(headers: Vec<&str>, rows: I) -> String
where
    I: IntoIterator<Item = Vec<String>>,
{
    let mut lines = vec![headers.into_iter().map(csv_cell).collect::<Vec<_>>().join(",")];
    lines.extend(rows.into_iter().map(|row| row.into_iter().map(csv_cell).collect::<Vec<_>>().join(",")));
    lines.join("\n")
}

fn csv_cell(value: impl AsRef<str>) -> String {
    let escaped = value.as_ref().replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn operator_user_id(current_user: &CurrentUser) -> Option<String> {
    if current_user.system {
        return None;
    }
    Some(current_user.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_user_id_skips_virtual_system_user() {
        let user = current_user(true);

        assert_eq!(operator_user_id(&user), None);
    }

    #[test]
    fn operator_user_id_keeps_database_user() {
        let user = current_user(false);

        assert_eq!(operator_user_id(&user), Some("user_1".to_owned()));
    }

    fn current_user(system: bool) -> CurrentUser {
        CurrentUser {
            id: "user_1".into(),
            username: "admin".into(),
            role: "admin".into(),
            group_codes: vec!["default".into()],
            system,
        }
    }
}
