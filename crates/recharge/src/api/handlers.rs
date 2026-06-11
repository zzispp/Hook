use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;

use serde::Deserialize;
use types::{
    pagination::PageRequest,
    recharge::{
        PaymentChannelResponse, PaymentChannelUpdatePayload, PublicPaymentChannelResponse, RechargeOrderCreatePayload, RechargeOrderCreateResponse,
        RechargeOrderDatePreset, RechargeOrderListFilters, RechargeOrderListResponse, RechargeOrderSummaryResponse, RechargePackageCreatePayload,
        RechargePackageListFilters, RechargePackageListResponse, RechargePackageResponse, RechargePackageUpdatePayload, UserRechargePackageListResponse,
    },
    response::ApiResponse,
};

use crate::api::{RechargeApiError, RechargeApiState};
use crate::application::RechargeError;

const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, RechargeApiError>;

#[derive(Debug, Deserialize)]
pub struct RechargePackageListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RechargeOrderListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub date_preset: RechargeOrderDatePreset,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub tz_offset_minutes: i32,
}

pub async fn list_packages(
    State(state): State<RechargeApiState>,
    Query(query): Query<RechargePackageListQuery>,
) -> ApiResult<ApiJson<RechargePackageListResponse>> {
    Ok(ok(state.recharge.list_packages((&query).into(), query.into()).await?))
}

pub async fn list_user_packages(
    State(state): State<RechargeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<UserRechargePackageListQuery>,
) -> ApiResult<ApiJson<UserRechargePackageListResponse>> {
    ensure_user_access(&current_user)?;
    Ok(ok(state.recharge.list_user_packages(query.into()).await?))
}

pub async fn create_package(
    State(state): State<RechargeApiState>,
    Json(payload): Json<RechargePackageCreatePayload>,
) -> ApiResult<ApiJson<RechargePackageResponse>> {
    Ok(ok(state.recharge.create_package(payload).await?.into()))
}

pub async fn update_package(
    State(state): State<RechargeApiState>,
    Path(id): Path<String>,
    Json(payload): Json<RechargePackageUpdatePayload>,
) -> ApiResult<ApiJson<RechargePackageResponse>> {
    Ok(ok(state.recharge.update_package(&id, payload).await?.into()))
}

pub async fn list_orders(State(state): State<RechargeApiState>, Query(query): Query<RechargeOrderListQuery>) -> ApiResult<ApiJson<RechargeOrderListResponse>> {
    Ok(ok(state.recharge.list_orders((&query).into(), query.into()).await?))
}

pub async fn list_order_summary(
    State(state): State<RechargeApiState>,
    Query(query): Query<RechargeOrderListQuery>,
) -> ApiResult<ApiJson<RechargeOrderSummaryResponse>> {
    Ok(ok(state.recharge.list_order_summary((&query).into(), query.into()).await?))
}

pub async fn list_user_orders(
    State(state): State<RechargeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<UserRechargeOrderListQuery>,
) -> ApiResult<ApiJson<RechargeOrderListResponse>> {
    ensure_user_access(&current_user)?;
    Ok(ok(state.recharge.list_user_orders(&current_user.id, query.into()).await?))
}

pub async fn create_user_order(
    State(state): State<RechargeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<RechargeOrderCreatePayload>,
) -> ApiResult<ApiJson<RechargeOrderCreateResponse>> {
    ensure_user_access(&current_user)?;
    verify_recharge_captcha(&state, payload.captcha_token.as_deref()).await?;
    Ok(ok(state.recharge.create_user_order(&current_user.id, payload).await?))
}

pub async fn list_payment_channels(State(state): State<RechargeApiState>) -> ApiResult<ApiJson<Vec<PaymentChannelResponse>>> {
    Ok(ok(state.recharge.list_payment_channels().await?.into_iter().map(Into::into).collect()))
}

pub async fn list_user_payment_channels(
    State(state): State<RechargeApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> ApiResult<ApiJson<Vec<PublicPaymentChannelResponse>>> {
    ensure_user_access(&current_user)?;
    Ok(ok(state.recharge.list_user_payment_channels().await?))
}

pub async fn update_payment_channel(
    State(state): State<RechargeApiState>,
    Path(code): Path<String>,
    Json(payload): Json<PaymentChannelUpdatePayload>,
) -> ApiResult<ApiJson<PaymentChannelResponse>> {
    Ok(ok(state.recharge.update_payment_channel(&code, payload).await?.into()))
}

impl From<&RechargePackageListQuery> for PageRequest {
    fn from(value: &RechargePackageListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<RechargePackageListQuery> for RechargePackageListFilters {
    fn from(value: RechargePackageListQuery) -> Self {
        Self {
            search: value.search,
            status: value.status,
        }
    }
}

impl From<&RechargeOrderListQuery> for PageRequest {
    fn from(value: &RechargeOrderListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserRechargePackageListQuery {
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct UserRechargeOrderListQuery {
    pub page: u64,
    pub page_size: u64,
}

impl From<UserRechargePackageListQuery> for PageRequest {
    fn from(value: UserRechargePackageListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<UserRechargeOrderListQuery> for PageRequest {
    fn from(value: UserRechargeOrderListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<RechargeOrderListQuery> for RechargeOrderListFilters {
    fn from(value: RechargeOrderListQuery) -> Self {
        Self {
            search: value.search,
            status: value.status,
            date_preset: value.date_preset,
            start_date: value.start_date,
            end_date: value.end_date,
            tz_offset_minutes: value.tz_offset_minutes,
            paid_at_start: None,
            paid_at_end: None,
        }
    }
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

fn ensure_user_access(current_user: &CurrentUser) -> ApiResult<()> {
    if current_user.role != USER_ROLE || current_user.system {
        return Err(RechargeError::Forbidden.into());
    }
    Ok(())
}

async fn verify_recharge_captcha(state: &RechargeApiState, token: Option<&str>) -> ApiResult<()> {
    state.captcha.verify_recharge(token).await.map_err(captcha_error)
}

fn captcha_error(error: captcha::application::CaptchaError) -> RechargeApiError {
    match error {
        captcha::application::CaptchaError::InvalidInput(message) => RechargeError::InvalidInput(message).into(),
        captcha::application::CaptchaError::Infrastructure(message) => RechargeError::Infrastructure(message).into(),
    }
}
