use axum::{
    Extension,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use rbac::api::CurrentUser;
use types::{
    pagination::PageSliceRequest,
    user::{
        AffiliateCommissionListResponse, AffiliateCommissionQuery, AffiliateReferralListResponse, AffiliateReferralQuery, AffiliateSummaryResponse, UserId,
    },
};

use crate::api::{ApiState, handlers::shared::*};

const CSV_CONTENT_DISPOSITION: &str = "attachment; filename=\"affiliate-commissions.csv\"";

pub async fn account_affiliate_summary(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> ApiResult<ApiJson<AffiliateSummaryResponse>> {
    Ok(ok(state.affiliates.affiliate_summary(UserId(current_user.id)).await?))
}

pub async fn account_affiliate_referrals(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AffiliateReferralQuery>,
) -> ApiResult<ApiJson<AffiliateReferralListResponse>> {
    let request = page_slice(query.page, query.page_size);
    let page = state.affiliates.list_affiliate_referrals(UserId(current_user.id), request, query).await?;
    Ok(ok(AffiliateReferralListResponse {
        items: page.items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }))
}

pub async fn account_affiliate_commissions(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AffiliateCommissionQuery>,
) -> ApiResult<ApiJson<AffiliateCommissionListResponse>> {
    let request = page_slice(query.page, query.page_size);
    let page = state.affiliates.list_affiliate_commissions(UserId(current_user.id), request, query).await?;
    Ok(ok(AffiliateCommissionListResponse {
        items: page.items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }))
}

pub async fn export_account_affiliate_commissions(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AffiliateCommissionQuery>,
) -> ApiResult<Response> {
    let items = state.affiliates.export_affiliate_commissions(UserId(current_user.id), query).await?;
    Ok(csv_response(commission_csv(items)))
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

fn commission_csv(items: Vec<types::user::AffiliateCommissionItem>) -> String {
    let rows = items.into_iter().map(|item| {
        vec![
            item.id,
            item.referred.referred_user_id,
            item.referred.username,
            item.referred.masked_email,
            item.recharge_order_no,
            item.payable_amount.to_string(),
            item.commission_percent.to_string(),
            item.commission_amount.to_string(),
            item.wallet_transaction_id.unwrap_or_default(),
            item.status,
            item.failure_reason.unwrap_or_default(),
            item.created_at,
        ]
    });
    csv(
        &[
            "id",
            "referred_user_id",
            "referred_username",
            "referred_email",
            "recharge_order_no",
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

fn csv<'a>(headers: &[&str], rows: impl IntoIterator<Item = Vec<String>>) -> String {
    let mut lines = vec![csv_row(headers.iter().map(|value| (*value).to_owned()).collect())];
    lines.extend(rows.into_iter().map(csv_row));
    lines.join("\n")
}

fn csv_row(values: Vec<String>) -> String {
    values.into_iter().map(csv_cell).collect::<Vec<_>>().join(",")
}

fn csv_cell(value: String) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
