use axum::{Json, extract::State};
use types::{
    captcha::{CaptchaChallengeResponse, CaptchaConfigResponse, CaptchaRedeemPayload, CaptchaRedeemResponse},
    response::ApiResponse,
};

use crate::api::{CaptchaApiError, CaptchaApiState};

type ApiResult<T> = Result<T, CaptchaApiError>;

pub async fn config(State(state): State<CaptchaApiState>) -> ApiResult<Json<ApiResponse<CaptchaConfigResponse>>> {
    Ok(Json(ApiResponse::new(state.captcha.config().await?)))
}

pub async fn challenge(State(state): State<CaptchaApiState>) -> ApiResult<Json<CaptchaChallengeResponse>> {
    Ok(Json(state.captcha.challenge().await?))
}

pub async fn redeem(State(state): State<CaptchaApiState>, Json(payload): Json<CaptchaRedeemPayload>) -> ApiResult<Json<CaptchaRedeemResponse>> {
    Ok(Json(state.captcha.redeem(payload).await?))
}
