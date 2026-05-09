use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;
use serde::Deserialize;
use types::{
    pagination::PageRequest,
    response::ApiResponse,
    wallet::{
        AdminWalletAdjustmentPayload, AdminWalletAdjustmentResponse, AdminWalletLedgerFilters, AdminWalletLedgerResponse, AdminWalletListFilters,
        AdminWalletListResponse, AdminWalletTransactionsResponse, WalletAdjustment, WalletBalanceResponse, WalletTransactionsResponse,
    },
};

use crate::api::{WalletApiError, WalletApiState};
use crate::application::WalletError;

const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, WalletApiError>;

#[derive(Debug, Deserialize)]
pub struct WalletListQuery {
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct AdminWalletListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminWalletLedgerQuery {
    pub page: u64,
    pub page_size: u64,
    pub category: Option<String>,
    pub reason_code: Option<String>,
    pub owner_type: Option<String>,
}

pub async fn balance(State(state): State<WalletApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<ApiJson<WalletBalanceResponse>> {
    ensure_user_wallet_access(&current_user)?;
    Ok(ok(state.wallets.balance(&current_user.id).await?))
}

pub async fn transactions(
    State(state): State<WalletApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<WalletListQuery>,
) -> ApiResult<ApiJson<WalletTransactionsResponse>> {
    ensure_user_wallet_access(&current_user)?;
    Ok(ok(state.wallets.transactions(&current_user.id, query.into()).await?))
}

pub async fn admin_wallets(State(state): State<WalletApiState>, Query(query): Query<AdminWalletListQuery>) -> ApiResult<ApiJson<AdminWalletListResponse>> {
    let page = PageRequest::from(&query);
    Ok(ok(state.wallets.admin_wallets(page, query.into()).await?))
}

pub async fn admin_ledger(State(state): State<WalletApiState>, Query(query): Query<AdminWalletLedgerQuery>) -> ApiResult<ApiJson<AdminWalletLedgerResponse>> {
    let page = PageRequest::from(&query);
    Ok(ok(state.wallets.admin_ledger(page, query.into()).await?))
}

pub async fn admin_transactions(
    State(state): State<WalletApiState>,
    Path(wallet_id): Path<String>,
    Query(query): Query<WalletListQuery>,
) -> ApiResult<ApiJson<AdminWalletTransactionsResponse>> {
    Ok(ok(state.wallets.admin_transactions(&wallet_id, query.into()).await?))
}

pub async fn admin_adjust_wallet(
    State(state): State<WalletApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(wallet_id): Path<String>,
    Json(payload): Json<AdminWalletAdjustmentPayload>,
) -> ApiResult<ApiJson<AdminWalletAdjustmentResponse>> {
    let transaction = state.wallets.adjust_wallet(adjustment(wallet_id, current_user.id, payload)).await?;
    Ok(ok(AdminWalletAdjustmentResponse {
        transaction: transaction.into(),
    }))
}

impl From<WalletListQuery> for PageRequest {
    fn from(value: WalletListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<&AdminWalletListQuery> for PageRequest {
    fn from(value: &AdminWalletListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<&AdminWalletLedgerQuery> for PageRequest {
    fn from(value: &AdminWalletLedgerQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<AdminWalletListQuery> for AdminWalletListFilters {
    fn from(value: AdminWalletListQuery) -> Self {
        Self {
            search: value.search,
            status: value.status,
        }
    }
}

impl From<AdminWalletLedgerQuery> for AdminWalletLedgerFilters {
    fn from(value: AdminWalletLedgerQuery) -> Self {
        Self {
            category: value.category,
            reason_code: value.reason_code,
            owner_type: value.owner_type,
        }
    }
}

fn adjustment(wallet_id: String, operator_id: String, payload: AdminWalletAdjustmentPayload) -> WalletAdjustment {
    WalletAdjustment {
        wallet_id,
        amount: payload.amount,
        balance_type: payload.balance_type.into(),
        adjustment_type: payload.adjustment_type.into(),
        operator_id: Some(operator_id),
        description: payload.description,
    }
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

fn ensure_user_wallet_access(current_user: &CurrentUser) -> ApiResult<()> {
    if current_user.role != USER_ROLE || current_user.system {
        return Err(WalletError::Forbidden.into());
    }
    Ok(())
}
