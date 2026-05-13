use axum::{
    Router,
    routing::{get, post},
};

use crate::api::{
    SettingApiState,
    handlers::{get_currency_display, get_exchange_rate, get_system_settings, test_smtp_connection, update_system_settings},
};

pub fn create_router(state: SettingApiState) -> Router {
    Router::new()
        .route("/settings/display-currency", get(get_currency_display))
        .route("/admin/settings/system", get(get_system_settings).patch(update_system_settings))
        .route("/admin/settings/smtp/test", post(test_smtp_connection))
        .route("/admin/settings/exchange-rate", get(get_exchange_rate))
        .with_state(state)
}
