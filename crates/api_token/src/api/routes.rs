use axum::{Router, routing::get};

use crate::api::{
    ApiTokenApiState,
    handlers::{
        admin_token_secret, create_admin_token, create_token, delete_admin_token, delete_token, get_admin_token, get_token, list_admin_tokens, list_tokens,
        token_secret, update_admin_token, update_token,
    },
};

pub fn create_router(state: ApiTokenApiState) -> Router {
    Router::new()
        .route("/tokens", get(list_tokens).post(create_token))
        .route("/tokens/{id}", get(get_token).patch(update_token).delete(delete_token))
        .route("/tokens/{id}/secret", get(token_secret))
        .route("/admin/tokens", get(list_admin_tokens).post(create_admin_token))
        .route("/admin/tokens/{id}", get(get_admin_token).patch(update_admin_token).delete(delete_admin_token))
        .route("/admin/tokens/{id}/secret", get(admin_token_secret))
        .with_state(state)
}
