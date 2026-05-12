use axum::{Json, extract::State};
use types::{
    response::ApiResponse,
    system_setting::{CurrencyDisplayResponse, DisplayCurrency, ExchangeRateResponse, SystemSettingsResponse, SystemSettingsUpdate},
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

pub async fn get_currency_display(State(state): State<SettingApiState>) -> ApiResult<ApiJson<CurrencyDisplayResponse>> {
    let settings = state.settings.get_system_settings().await?;
    let currency = settings.currency;
    let usd_cny_rate = if currency == DisplayCurrency::Cny {
        Some(
            state
                .exchange_rates
                .usd_cny_rate()
                .await
                .map_err(|message| SettingApiError(crate::application::SettingError::Infrastructure(message)))?,
        )
    } else {
        None
    };

    Ok(ok(CurrencyDisplayResponse { currency, usd_cny_rate }))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
