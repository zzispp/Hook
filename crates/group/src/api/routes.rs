use axum::{Router, routing::get};

use crate::api::{
    GroupApiState,
    handlers::{available_groups, create_group, delete_group, get_group, list_groups, update_group},
};

pub fn create_router(state: GroupApiState) -> Router {
    Router::new()
        .route("/admin/groups", get(list_groups).post(create_group))
        .route("/admin/groups/{id}", get(get_group).patch(update_group).delete(delete_group))
        .route("/groups/available", get(available_groups))
        .with_state(state)
}
