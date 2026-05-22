use axum::{Router, routing::get};

use crate::api::{
    SchedulerApiState,
    handlers::{list_runs, list_tasks, update_task},
};

pub fn create_router(state: SchedulerApiState) -> Router {
    Router::new()
        .route("/admin/scheduled-tasks", get(list_tasks))
        .route("/admin/scheduled-tasks/{code}", axum::routing::patch(update_task))
        .route("/admin/scheduled-task-runs", get(list_runs))
        .with_state(state)
}
