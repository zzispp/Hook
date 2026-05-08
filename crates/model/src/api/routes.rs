use axum::{
    Router,
    routing::{get, post},
};

use crate::api::{
    ModelApiState,
    handlers::{
        batch_delete_global_models, catalog, create_global_model, delete_global_model, external_models, get_global_model, global_model_providers,
        list_global_models, public_catalog, update_global_model,
    },
};

pub fn create_router(state: ModelApiState) -> Router {
    Router::new()
        .route("/admin/models/global", get(list_global_models).post(create_global_model))
        .route("/admin/models/global/batch-delete", post(batch_delete_global_models))
        .route(
            "/admin/models/global/{id}",
            get(get_global_model).patch(update_global_model).delete(delete_global_model),
        )
        .route("/admin/models/global/{id}/providers", get(global_model_providers))
        .route("/admin/models/catalog", get(catalog))
        .route("/admin/models/external", get(external_models))
        .route("/models/catalog", get(public_catalog))
        .with_state(state)
}
