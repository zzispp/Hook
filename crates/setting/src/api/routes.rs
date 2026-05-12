use axum::{Router, routing::get};

use crate::api::{
    SettingApiState,
    handlers::{get_exchange_rate, get_system_settings, update_system_settings},
};

pub fn create_router(state: SettingApiState) -> Router {
    Router::new()
        .route("/admin/settings/system", get(get_system_settings).patch(update_system_settings))
        .route("/admin/settings/exchange-rate", get(get_exchange_rate))
        .with_state(state)
}
