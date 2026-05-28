use axum::{Router, routing::get};

use crate::api::{
    DashboardApiState,
    handlers::{activity, filter_options, overview, user_stats_leaderboard, user_stats_time_series, user_usage_stats},
};

pub fn create_router(state: DashboardApiState) -> Router {
    Router::new()
        .route("/dashboard/overview", get(overview))
        .route("/dashboard/activity", get(activity))
        .route("/dashboard/filter-options", get(filter_options))
        .route("/admin/stats/leaderboard/users", get(user_stats_leaderboard))
        .route("/admin/usage/stats", get(user_usage_stats))
        .route("/admin/stats/time-series", get(user_stats_time_series))
        .with_state(state)
}
