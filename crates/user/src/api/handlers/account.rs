use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;
use types::user::{
    AccountPasswordChangePayload, AccountPasswordEmailCodePayload, AccountProfileResponse, AccountProviderLinkResponse, OAuthCallbackQuery, OAuthStartResponse,
    UserId, UserIdentitySummary, UserResponse, WalletSignInPayload,
};

use crate::api::{ApiState, handlers::shared::*};
use crate::application::WalletSignInInput;

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

pub async fn account_oauth_start(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(provider): Path<String>,
) -> ApiResult<ApiJson<OAuthStartResponse>> {
    let provider = parse_provider(&provider)?;
    let authorization_url = state.users.account_oauth_start(UserId(current_user.id), provider).await?;
    Ok(ok(OAuthStartResponse { authorization_url }))
}

pub async fn account_oauth_callback(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> ApiResult<ApiJson<AccountProviderLinkResponse>> {
    let provider = parse_provider(&provider)?;
    Ok(ok(state
        .users
        .account_oauth_callback(UserId(current_user.id), provider, query.code, query.state)
        .await?))
}

pub async fn account_wallet_link(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<WalletSignInPayload>,
) -> ApiResult<ApiJson<AccountProviderLinkResponse>> {
    Ok(ok(state
        .users
        .account_wallet_link(
            UserId(current_user.id),
            WalletSignInInput {
                provider: payload.provider,
                address: payload.address,
                message: payload.message,
                signature: payload.signature,
                chain_id: payload.chain_id,
            },
        )
        .await?))
}
