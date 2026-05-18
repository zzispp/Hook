use axum::{Router, routing::get};

use crate::api::{
    DashboardApiState,
    handlers::{activity, filter_options, overview},
};

pub fn create_router(state: DashboardApiState) -> Router {
    Router::new()
        .route("/dashboard/overview", get(overview))
        .route("/dashboard/activity", get(activity))
        .route("/dashboard/filter-options", get(filter_options))
        .with_state(state)
}
