use axum::{Json, extract::State};
use types::{
    response::ApiResponse,
    system_setting::{ExchangeRateResponse, SystemSettingsResponse, SystemSettingsUpdate},
};

use crate::api::{SettingApiError, SettingApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, SettingApiError>;

pub async fn get_system_settings(State(state): State<SettingApiState>) -> ApiResult<ApiJson<SystemSettingsResponse>> {
    Ok(ok(state.settings.get_system_settings().await?))
}

pub async fn update_system_settings(
    State(state): State<SettingApiState>,
    Json(payload): Json<SystemSettingsUpdate>,
) -> ApiResult<ApiJson<SystemSettingsResponse>> {
    Ok(ok(state.settings.update_system_settings(payload).await?))
}

pub async fn get_exchange_rate(State(state): State<SettingApiState>) -> ApiResult<ApiJson<ExchangeRateResponse>> {
    state
        .exchange_rates
        .usd_cny_rate()
        .await
        .map(ok)
        .map_err(|message| SettingApiError(crate::application::SettingError::Infrastructure(message)))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
