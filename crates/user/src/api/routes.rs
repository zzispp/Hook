use axum::{
    Router,
    routing::{get, post, put},
};

use crate::api::{
    ApiState,
    handlers::{create_user, delete_user, health, list_users, me, refresh, replace_user, sign_in, sign_up},
};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/auth/sign-up", post(sign_up))
        .route("/api/auth/sign-in", post(sign_in))
        .route("/api/auth/refresh", post(refresh))
        .route("/api/auth/me", get(me))
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/{id}", put(replace_user).delete(delete_user))
        .with_state(state)
}

#[cfg(test)]
mod tests;
