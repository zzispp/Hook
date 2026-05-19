use axum::{
    Router,
    routing::{get, post, put},
};

use crate::api::{
    ApiState,
    handlers::{
        auth_config, create_user, delete_user, list_users, me, refresh, replace_user, request_password_reset, request_registration_email_code, reset_password,
        sign_in, sign_up,
    },
};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/auth/config", get(auth_config))
        .route("/auth/registration-email-code", post(request_registration_email_code))
        .route("/auth/sign-up", post(sign_up))
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/refresh", post(refresh))
        .route("/auth/password-reset/request", post(request_password_reset))
        .route("/auth/password-reset/confirm", post(reset_password))
        .route("/auth/me", get(me))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", put(replace_user).delete(delete_user))
        .with_state(state)
}

#[cfg(test)]
mod tests;
