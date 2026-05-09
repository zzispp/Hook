use axum::{
    Extension, Json,
    extract::{Query, State},
};
use rbac::api::CurrentUser;
use serde::Deserialize;
use types::{
    pagination::PageRequest,
    response::ApiResponse,
    wallet::{WalletBalanceResponse, WalletTransactionsResponse},
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

impl From<WalletListQuery> for PageRequest {
    fn from(value: WalletListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
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
