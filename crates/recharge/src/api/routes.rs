use axum::{
    Router,
    routing::{get, patch},
};

use crate::api::{
    RechargeApiState,
    handlers::{
        create_package, create_user_order, list_orders, list_packages, list_payment_channels, list_user_orders, list_user_packages, update_package,
        update_payment_channel,
    },
};

pub fn create_router(state: RechargeApiState) -> Router {
    Router::new()
        .route("/recharge-packages", get(list_user_packages))
        .route("/recharge-orders", get(list_user_orders).post(create_user_order))
        .route("/admin/recharge-packages", get(list_packages).post(create_package))
        .route("/admin/recharge-packages/{id}", patch(update_package))
        .route("/admin/recharge-orders", get(list_orders))
        .route("/admin/payment-channels", get(list_payment_channels))
        .route("/admin/payment-channels/{code}", patch(update_payment_channel))
        .with_state(state)
}
