use axum::{Router, routing::get};

use crate::api::{
    DashboardApiState,
    handlers::{
        activity, api_key_leaderboard, cost_forecast, cost_savings, filter_options, overview, provider_aggregation, user_stats_leaderboard,
        user_stats_time_series, user_usage_stats,
    },
};

pub fn create_router(state: DashboardApiState) -> Router {
    Router::new()
        .route("/dashboard/overview", get(overview))
        .route("/dashboard/activity", get(activity))
        .route("/dashboard/filter-options", get(filter_options))
        .route("/admin/stats/leaderboard/users", get(user_stats_leaderboard))
        .route("/admin/stats/leaderboard/api-keys", get(api_key_leaderboard))
        .route("/admin/stats/cost/forecast", get(cost_forecast))
        .route("/admin/stats/cost/savings", get(cost_savings))
        .route("/admin/usage/aggregation/stats", get(provider_aggregation))
        .route("/admin/usage/stats", get(user_usage_stats))
        .route("/admin/stats/time-series", get(user_stats_time_series))
        .with_state(state)
}
