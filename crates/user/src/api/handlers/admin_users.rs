use axum::{
    Json,
    extract::{Path, Query, State},
};
use types::{
    pagination::PageRequest,
    user::{ListUsersQuery, User, UserId, UserListFilters, UserPayload, UserResponse, UsersPageResponse},
};

use crate::api::{ApiState, handlers::shared::*};

pub async fn create_user(State(state): State<ApiState>, Json(payload): Json<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.create_user(payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn replace_user(State(state): State<ApiState>, Path(id): Path<String>, Json(payload): Json<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.replace_user(UserId(id), payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn delete_user(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.users.delete_user(UserId(id)).await?;
    Ok(ok(()))
}

pub async fn get_user(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.admin_user(UserId(id)).await?;
    let identities = state.users.identities(user.id.clone()).await?;
    Ok(ok(UserResponse::from(user).with_identities(identities)))
}

pub async fn admin_unlink_identity(State(state): State<ApiState>, Path((user_id, identity_id)): Path<(String, String)>) -> ApiResult<ApiJson<()>> {
    state.users.admin_unlink_identity(UserId(user_id), identity_id).await?;
    Ok(ok(()))
}

pub async fn list_users(State(state): State<ApiState>, Query(query): Query<ListUsersQuery>) -> ApiResult<ApiJson<UsersPageResponse>> {
    let page = state.users.list_users(page_request(&query), user_filters(query)).await?;
    let user_ids = user_ids(&page.items);
    let wallets = state.users.wallet_summaries(&user_ids).await?;
    let identities = state.users.identity_summaries(&user_ids).await?;
    let response = UsersPageResponse {
        items: page
            .items
            .into_iter()
            .map(|user| {
                let wallet = wallets.get(&user.id.0).cloned();
                let identity_items = identities.get(&user.id.0).cloned().unwrap_or_default();
                UserResponse::from(user).with_wallet(wallet).with_identities(identity_items)
            })
            .collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    };
    Ok(ok(response))
}

fn page_request(query: &ListUsersQuery) -> PageRequest {
    PageRequest {
        page: query.page,
        page_size: query.page_size,
    }
}

fn user_filters(query: ListUsersQuery) -> UserListFilters {
    UserListFilters {
        search: query.search,
        role: query.role,
        group_code: query.group_code,
        is_active: query.is_active,
    }
}

fn user_ids(users: &[User]) -> Vec<String> {
    users.iter().filter(|user| !user.system).map(|user| user.id.0.clone()).collect()
}
