use axum::{Router, routing::get};

use crate::api::{
    ModelStatusApiState,
    handlers::{batch_create_checks, batch_delete_checks, batch_update_checks, create_check, delete_check, list_admin, list_public, list_runs, update_check},
};

pub fn create_router(state: ModelStatusApiState) -> Router {
    Router::new()
        .route("/model-status/checks", get(list_public))
        .route("/admin/model-status/checks", get(list_admin).post(create_check))
        .route("/admin/model-status/checks/batch-create", axum::routing::post(batch_create_checks))
        .route("/admin/model-status/checks/batch-delete", axum::routing::post(batch_delete_checks))
        .route("/admin/model-status/checks/batch-update", axum::routing::post(batch_update_checks))
        .route("/admin/model-status/runs", get(list_runs))
        .route("/admin/model-status/checks/{id}", axum::routing::patch(update_check).delete(delete_check))
        .with_state(state)
}
