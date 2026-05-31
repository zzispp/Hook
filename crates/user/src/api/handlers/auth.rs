use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use constants::auth::{DEFAULT_USER_IS_ACTIVE, DEFAULT_USER_ROLE};
use types::user::{
    AuthConfigResponse, NewUser, OAuthBindExistingPayload, OAuthCallbackQuery, OAuthCallbackResponse, OAuthStartResponse, PasswordResetConfirmPayload,
    PasswordResetRequestPayload, RefreshTokenPayload, RegistrationEmailCodePayload, SignInPayload, SignUpPayload, SignUpUser, USER_QUOTA_MODE_WALLET,
    WalletCompletePayload, WalletEmailCodePayload, WalletNoncePayload, WalletNonceResponse, WalletSignInPayload, WalletSignInResponse,
};

use crate::api::{ApiState, handlers::shared::*};
use crate::application::{AppError, WalletNonceInput, WalletSignInInput};

pub async fn sign_up(State(state): State<ApiState>, Json(payload): Json<SignUpPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    verify_registration_captcha(&state, payload.captcha_token.as_deref()).await?;
    let user = state.users.sign_up(new_sign_up_user(payload)).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn auth_config(State(state): State<ApiState>) -> ApiResult<ApiJson<AuthConfigResponse>> {
    Ok(ok(state.users.auth_config().await?))
}

pub async fn request_registration_email_code(State(state): State<ApiState>, Json(payload): Json<RegistrationEmailCodePayload>) -> ApiResult<ApiJson<()>> {
    state.users.request_registration_email_code(payload.into()).await?;
    Ok(ok(()))
}

pub async fn sign_in(State(state): State<ApiState>, Json(payload): Json<SignInPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    verify_login_captcha(&state, payload.captcha_token.as_deref()).await?;
    let user = state.users.sign_in(payload.into()).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn oauth_start(State(state): State<ApiState>, Path(provider): Path<String>) -> ApiResult<ApiJson<OAuthStartResponse>> {
    let provider = parse_provider(&provider)?;
    let authorization_url = state.users.oauth_start(provider).await?;
    Ok(ok(OAuthStartResponse { authorization_url }))
}

pub async fn oauth_callback(
    State(state): State<ApiState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> ApiResult<ApiJson<OAuthCallbackResponse>> {
    let provider = parse_provider(&provider)?;
    let result = state.users.oauth_callback(provider, query.code, query.state).await?;
    match result {
        crate::application::OAuthSignInResult::Authenticated(user) => {
            let user = *user;
            let tokens = state.tokens.issue_pair(user.id.clone())?;
            Ok(ok(OAuthCallbackResponse::Authenticated(Box::new(new_auth_session_data(user.into(), tokens)))))
        }
        crate::application::OAuthSignInResult::BindingRequired {
            ticket,
            provider,
            email,
            username,
        } => Ok(ok(OAuthCallbackResponse::BindingRequired {
            binding_ticket: ticket,
            provider,
            email,
            username,
        })),
    }
}

pub async fn bind_oauth_existing(
    State(state): State<ApiState>,
    Path(provider): Path<String>,
    Json(payload): Json<OAuthBindExistingPayload>,
) -> ApiResult<ApiJson<AuthSessionResponse>> {
    let provider = parse_provider(&provider)?;
    let user = state.users.bind_oauth_existing(provider, payload.binding_ticket).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn wallet_nonce(State(state): State<ApiState>, Json(payload): Json<WalletNoncePayload>) -> ApiResult<ApiJson<WalletNonceResponse>> {
    let challenge = state
        .users
        .wallet_nonce(WalletNonceInput {
            provider: payload.provider,
            address: payload.address,
            chain_id: payload.chain_id,
        })
        .await?;
    Ok(ok(WalletNonceResponse {
        message: challenge.message,
        nonce: challenge.nonce,
    }))
}

pub async fn wallet_sign_in(State(state): State<ApiState>, Json(payload): Json<WalletSignInPayload>) -> ApiResult<ApiJson<WalletSignInResponse>> {
    let result = state
        .users
        .wallet_sign_in(WalletSignInInput {
            provider: payload.provider,
            address: payload.address,
            message: payload.message,
            signature: payload.signature,
            chain_id: payload.chain_id,
        })
        .await?;
    match result {
        crate::application::WalletSignInResult::Authenticated(user) => {
            let user = *user;
            let tokens = state.tokens.issue_pair(user.id.clone())?;
            Ok(ok(WalletSignInResponse::Authenticated(Box::new(new_auth_session_data(user.into(), tokens)))))
        }
        crate::application::WalletSignInResult::EmailRequired { ticket, provider, address } => Ok(ok(WalletSignInResponse::EmailRequired {
            wallet_ticket: ticket,
            provider,
            address,
        })),
    }
}

pub async fn wallet_email_code(State(state): State<ApiState>, Json(payload): Json<WalletEmailCodePayload>) -> ApiResult<ApiJson<()>> {
    state
        .users
        .request_wallet_email_code(payload.wallet_ticket, payload.email, payload.lang)
        .await?;
    Ok(ok(()))
}

pub async fn wallet_complete(State(state): State<ApiState>, Json(payload): Json<WalletCompletePayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    let user = state
        .users
        .complete_wallet(payload.wallet_ticket, payload.email, payload.email_verification_code)
        .await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn refresh(State(state): State<ApiState>, Json(payload): Json<RefreshTokenPayload>) -> ApiResult<ApiJson<TokenPairResponse>> {
    let (user_id, tokens) = state.tokens.refresh(&payload.refresh_token)?;
    state.users.authenticated_user(user_id).await?;
    Ok(ok(tokens.into()))
}

pub async fn request_password_reset(State(state): State<ApiState>, Json(payload): Json<PasswordResetRequestPayload>) -> ApiResult<ApiJson<()>> {
    state.users.request_password_reset(payload.into()).await?;
    Ok(ok(()))
}

pub async fn reset_password(State(state): State<ApiState>, Json(payload): Json<PasswordResetConfirmPayload>) -> ApiResult<ApiJson<()>> {
    state.users.reset_password(payload.into()).await?;
    Ok(ok(()))
}

pub async fn me(State(state): State<ApiState>, headers: HeaderMap) -> ApiResult<ApiJson<MeResponse>> {
    let access_token = bearer_token(&headers)?;
    let user_id = state.tokens.validate_access(access_token)?;
    let user = state.users.authenticated_user(user_id).await?;
    Ok(ok(MeResponse { user: user.into() }))
}

fn new_sign_up_user(payload: SignUpPayload) -> SignUpUser {
    SignUpUser {
        user: NewUser {
            username: payload.username,
            password: payload.password,
            email: payload.email,
            role: DEFAULT_USER_ROLE.into(),
            group_codes: None,
            is_active: DEFAULT_USER_IS_ACTIVE,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            rate_limit_rpm: None,
            quota_mode: USER_QUOTA_MODE_WALLET.into(),
        },
        email_verification_code: payload.email_verification_code,
    }
}

async fn verify_login_captcha(state: &ApiState, token: Option<&str>) -> ApiResult<()> {
    state.captcha.verify_login(token).await.map_err(captcha_error)
}

async fn verify_registration_captcha(state: &ApiState, token: Option<&str>) -> ApiResult<()> {
    state.captcha.verify_registration(token).await.map_err(captcha_error)
}

fn captcha_error(error: captcha::application::CaptchaError) -> crate::api::error::ApiError {
    match error {
        captcha::application::CaptchaError::InvalidInput(message) => crate::api::error::ApiError(AppError::InvalidInput(message)),
        captcha::application::CaptchaError::Infrastructure(message) => crate::api::error::ApiError(AppError::Infrastructure(message)),
    }
}
