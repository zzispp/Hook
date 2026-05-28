use std::collections::BTreeMap;

use axum::{
    Form, Json,
    extract::{Path, Query, State},
    response::Redirect,
};
use payment::{PaymentCallbackRequest, PaymentChannelConfig};
use serde::Deserialize;
use types::{
    pagination::PageRequest,
    recharge::{PaymentCallbackListFilters, PaymentCallbackRecordListResponse},
    response::ApiResponse,
};

use crate::{
    api::{RechargeApiError, RechargeApiState},
    application::{PaymentCallbackKind, RechargePaymentCallbackRequest},
};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, RechargeApiError>;

#[derive(Debug, Deserialize)]
pub struct PaymentCallbackListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub status: Option<String>,
}

pub async fn list_payment_callbacks(
    State(state): State<RechargeApiState>,
    Query(query): Query<PaymentCallbackListQuery>,
) -> ApiResult<ApiJson<PaymentCallbackRecordListResponse>> {
    Ok(Json(ApiResponse::new(
        state.recharge.list_payment_callbacks((&query).into(), query.into()).await?,
    )))
}

pub async fn handle_payment_notify_query(
    State(state): State<RechargeApiState>,
    Path(code): Path<String>,
    Query(params): Query<BTreeMap<String, String>>,
) -> ApiResult<String> {
    payment_notify(
        callback_input(CallbackParts {
            channel_code: code,
            callback_kind: PaymentCallbackKind::Notify,
            http_method: "GET".into(),
            params,
        }),
        state,
    )
    .await
}

pub async fn handle_payment_notify_form(
    State(state): State<RechargeApiState>,
    Path(code): Path<String>,
    Form(params): Form<BTreeMap<String, String>>,
) -> ApiResult<String> {
    payment_notify(
        callback_input(CallbackParts {
            channel_code: code,
            callback_kind: PaymentCallbackKind::Notify,
            http_method: "POST".into(),
            params,
        }),
        state,
    )
    .await
}

pub async fn handle_payment_return_query(
    State(state): State<RechargeApiState>,
    Path(code): Path<String>,
    Query(params): Query<BTreeMap<String, String>>,
) -> ApiResult<Redirect> {
    payment_return(
        callback_input(CallbackParts {
            channel_code: code,
            callback_kind: PaymentCallbackKind::Return,
            http_method: "GET".into(),
            params,
        }),
        state,
    )
    .await
}

pub async fn handle_payment_return_form(
    State(state): State<RechargeApiState>,
    Path(code): Path<String>,
    Form(params): Form<BTreeMap<String, String>>,
) -> ApiResult<Redirect> {
    payment_return(
        callback_input(CallbackParts {
            channel_code: code,
            callback_kind: PaymentCallbackKind::Return,
            http_method: "POST".into(),
            params,
        }),
        state,
    )
    .await
}

impl From<&PaymentCallbackListQuery> for PageRequest {
    fn from(value: &PaymentCallbackListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<PaymentCallbackListQuery> for PaymentCallbackListFilters {
    fn from(value: PaymentCallbackListQuery) -> Self {
        Self {
            search: value.search,
            status: value.status,
        }
    }
}

async fn payment_notify(input: RechargePaymentCallbackRequest, state: RechargeApiState) -> ApiResult<String> {
    Ok(state.recharge.handle_payment_callback(input).await?.response_body)
}

async fn payment_return(input: RechargePaymentCallbackRequest, state: RechargeApiState) -> ApiResult<Redirect> {
    state.recharge.handle_payment_callback(input).await?;
    Ok(Redirect::to("/dashboard/wallet"))
}

struct CallbackParts {
    channel_code: String,
    callback_kind: PaymentCallbackKind,
    http_method: String,
    params: BTreeMap<String, String>,
}

fn callback_input(parts: CallbackParts) -> RechargePaymentCallbackRequest {
    RechargePaymentCallbackRequest {
        channel_code: parts.channel_code,
        callback_kind: parts.callback_kind,
        http_method: parts.http_method,
        payment: PaymentCallbackRequest {
            params: parts.params,
            config: PaymentChannelConfig {
                config: serde_json::json!({}),
                secret: None,
            },
        },
    }
}
