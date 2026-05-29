use axum::{
    Extension, Json,
    extract::{Path, State},
};
use rbac::api::CurrentUser;
use types::user::{AccountPasswordChangePayload, AccountPasswordEmailCodePayload, AccountProfileResponse, UserId, UserIdentitySummary, UserResponse};

use crate::api::{ApiState, handlers::shared::*};

pub async fn account_profile(State(state): State<ApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<ApiJson<AccountProfileResponse>> {
    let user = state.users.profile(UserId(current_user.id)).await?;
    let identities = state.users.identities(user.id.clone()).await?;
    Ok(ok(AccountProfileResponse {
        user: UserResponse::from(user).with_identities(identities),
    }))
}

pub async fn account_password_email_code(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AccountPasswordEmailCodePayload>,
) -> ApiResult<ApiJson<()>> {
    state.users.request_account_password_email_code(UserId(current_user.id), payload).await?;
    Ok(ok(()))
}

pub async fn account_password_change(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AccountPasswordChangePayload>,
) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.change_account_password(UserId(current_user.id), payload).await?;
    let identities = state.users.identities(user.id.clone()).await?;
    Ok(ok(UserResponse::from(user).with_identities(identities)))
}

pub async fn account_identities(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> ApiResult<ApiJson<Vec<UserIdentitySummary>>> {
    Ok(ok(state.users.identities(UserId(current_user.id)).await?))
}

pub async fn account_unlink_identity(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(identity_id): Path<String>,
) -> ApiResult<ApiJson<()>> {
    state.users.unlink_identity(UserId(current_user.id), identity_id).await?;
    Ok(ok(()))
}
