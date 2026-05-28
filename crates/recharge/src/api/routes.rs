use axum::{
    Router,
    routing::{get, patch},
};

use crate::api::{
    RechargeApiState,
    handlers::{
        create_package, create_user_order, list_orders, list_packages, list_payment_channels, list_user_orders, list_user_packages, list_user_payment_channels,
        update_package, update_payment_channel,
    },
    payment_callbacks::{
        handle_payment_notify_form, handle_payment_notify_query, handle_payment_return_form, handle_payment_return_query, list_payment_callbacks,
    },
};

pub fn create_router(state: RechargeApiState) -> Router {
    Router::new()
        .route("/recharge-packages", get(list_user_packages))
        .route("/recharge-orders", get(list_user_orders).post(create_user_order))
        .route("/payment-channels", get(list_user_payment_channels))
        .route("/admin/recharge-packages", get(list_packages).post(create_package))
        .route("/admin/recharge-packages/{id}", patch(update_package))
        .route("/admin/recharge-orders", get(list_orders))
        .route("/admin/payment-callbacks", get(list_payment_callbacks))
        .route("/admin/payment-channels", get(list_payment_channels))
        .route("/admin/payment-channels/{code}", patch(update_payment_channel))
        .route("/payment/{code}/notify", get(handle_payment_notify_query).post(handle_payment_notify_form))
        .route("/payment/{code}/notify/", get(handle_payment_notify_query).post(handle_payment_notify_form))
        .route("/payment/{code}/return", get(handle_payment_return_query).post(handle_payment_return_form))
        .route("/payment/{code}/return/", get(handle_payment_return_query).post(handle_payment_return_form))
        .with_state(state)
}

#[cfg(test)]
#[path = "routes_tests.rs"]
mod tests;
