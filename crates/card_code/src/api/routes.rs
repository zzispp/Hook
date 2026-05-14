use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::api::{
    CardCodeApiState,
    handlers::{admin_batch_status, admin_create_type, admin_generate_codes, admin_list_codes, admin_list_types, admin_update_type, redeem_code},
};

pub fn create_router(state: CardCodeApiState) -> Router {
    Router::new()
        .route("/card-codes/redeem", post(redeem_code))
        .route("/admin/card-codes", get(admin_list_codes))
        .route("/admin/card-codes/generate", post(admin_generate_codes))
        .route("/admin/card-codes/batch-status", post(admin_batch_status))
        .route("/admin/card-code-types", get(admin_list_types).post(admin_create_type))
        .route("/admin/card-code-types/{id}", patch(admin_update_type))
        .with_state(state)
}
