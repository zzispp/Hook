use axum::{
    Extension, Json,
    extract::{ConnectInfo, Path, Query, State},
    http::HeaderMap,
};
use rbac::api::CurrentUser;
use serde::Deserialize;
use std::net::SocketAddr;
use types::{
    card_code::{
        CardCodeBatchStatusPayload, CardCodeBatchStatusResponse, CardCodeGeneratePayload, CardCodeGenerateResponse, CardCodeListFilters, CardCodeListResponse,
        CardCodeRedeemPayload, CardCodeRedeemResponse, CardCodeTypeCreatePayload, CardCodeTypeListFilters, CardCodeTypeListResponse, CardCodeTypeResponse,
        CardCodeTypeUpdatePayload,
    },
    pagination::PageRequest,
    response::ApiResponse,
};

use crate::{
    api::{CardCodeApiError, CardCodeApiState},
    application::{CardCodeOperator, CardCodeRedeemer},
};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, CardCodeApiError>;

#[derive(Debug, Deserialize)]
pub struct CardCodeTypeListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CardCodeListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub status: Option<String>,
    pub type_id: Option<String>,
}

pub async fn admin_list_types(
    State(state): State<CardCodeApiState>,
    Query(query): Query<CardCodeTypeListQuery>,
) -> ApiResult<ApiJson<CardCodeTypeListResponse>> {
    let page = PageRequest::from(&query);
    Ok(ok(state.card_codes.list_types(page, query.into()).await?))
}

pub async fn admin_create_type(
    State(state): State<CardCodeApiState>,
    Json(payload): Json<CardCodeTypeCreatePayload>,
) -> ApiResult<ApiJson<CardCodeTypeResponse>> {
    Ok(ok(state.card_codes.create_type(payload).await?.into()))
}

pub async fn admin_update_type(
    State(state): State<CardCodeApiState>,
    Path(id): Path<String>,
    Json(payload): Json<CardCodeTypeUpdatePayload>,
) -> ApiResult<ApiJson<CardCodeTypeResponse>> {
    Ok(ok(state.card_codes.update_type(&id, payload).await?.into()))
}

pub async fn admin_list_codes(State(state): State<CardCodeApiState>, Query(query): Query<CardCodeListQuery>) -> ApiResult<ApiJson<CardCodeListResponse>> {
    let page = PageRequest::from(&query);
    Ok(ok(state.card_codes.list_codes(page, query.into()).await?))
}

pub async fn admin_generate_codes(
    State(state): State<CardCodeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<CardCodeGeneratePayload>,
) -> ApiResult<ApiJson<CardCodeGenerateResponse>> {
    Ok(ok(state
        .card_codes
        .generate_codes(payload, operator(current_user, headers, remote_addr))
        .await?))
}

pub async fn admin_batch_status(
    State(state): State<CardCodeApiState>,
    Json(payload): Json<CardCodeBatchStatusPayload>,
) -> ApiResult<ApiJson<CardCodeBatchStatusResponse>> {
    Ok(ok(state.card_codes.batch_update_code_status(payload).await?))
}

pub async fn redeem_code(
    State(state): State<CardCodeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<CardCodeRedeemPayload>,
) -> ApiResult<ApiJson<CardCodeRedeemResponse>> {
    Ok(ok(state.card_codes.redeem(payload, redeemer(current_user, headers, remote_addr)).await?))
}

impl From<&CardCodeTypeListQuery> for PageRequest {
    fn from(value: &CardCodeTypeListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<CardCodeTypeListQuery> for CardCodeTypeListFilters {
    fn from(value: CardCodeTypeListQuery) -> Self {
        Self {
            search: value.search,
            status: value.status,
        }
    }
}

impl From<&CardCodeListQuery> for PageRequest {
    fn from(value: &CardCodeListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<CardCodeListQuery> for CardCodeListFilters {
    fn from(value: CardCodeListQuery) -> Self {
        Self {
            search: value.search,
            status: value.status,
            type_id: value.type_id,
        }
    }
}

fn operator(current_user: CurrentUser, headers: HeaderMap, remote_addr: SocketAddr) -> CardCodeOperator {
    CardCodeOperator {
        user_id: operator_user_id(&current_user),
        username: Some(current_user.username),
        client_ip: client_ip(&headers, remote_addr),
    }
}

fn operator_user_id(current_user: &CurrentUser) -> Option<String> {
    if current_user.system {
        return None;
    }
    Some(current_user.id.clone())
}

fn redeemer(current_user: CurrentUser, headers: HeaderMap, remote_addr: SocketAddr) -> CardCodeRedeemer {
    CardCodeRedeemer {
        user_id: current_user.id,
        username: current_user.username,
        client_ip: client_ip(&headers, remote_addr),
    }
}

fn client_ip(headers: &HeaderMap, remote_addr: SocketAddr) -> Option<String> {
    forwarded_ip(headers)
        .or_else(|| header_string(headers, "x-real-ip"))
        .or_else(|| Some(remote_addr.ip().to_string()))
}

fn forwarded_ip(headers: &HeaderMap) -> Option<String> {
    let value = header_string(headers, "x-forwarded-for")?;
    value.split(',').next().map(str::trim).filter(|item| !item.is_empty()).map(str::to_owned)
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(name).and_then(|value| value.to_str().ok()).map(str::to_owned)
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
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
            system,
        }
    }
}
