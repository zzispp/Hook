use axum::{
    Extension, Json,
    extract::{Query, State},
};
use rbac::api::CurrentUser;
use types::{
    dashboard::{
        DashboardActivityRequest, DashboardActivityResponse, DashboardApiKeyLeaderboardRequest, DashboardApiKeyLeaderboardResponse,
        DashboardCostForecastRequest, DashboardCostForecastResponse, DashboardCostSavingsRequest, DashboardCostSavingsResponse, DashboardFilterOptionsRequest,
        DashboardFilterOptionsResponse, DashboardOverviewRequest, DashboardOverviewResponse, DashboardProviderAggregationItem,
        DashboardProviderAggregationRequest, DashboardUserStatsLeaderboardRequest, DashboardUserStatsLeaderboardResponse, DashboardUserStatsTimeSeriesPoint,
        DashboardUserStatsTimeSeriesRequest, DashboardUserUsageStatsRequest, DashboardUserUsageStatsResponse,
    },
    response::ApiResponse,
};

use crate::{
    api::{DashboardApiError, DashboardApiState},
    application::DashboardActor,
};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, DashboardApiError>;

pub async fn overview(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardOverviewRequest>,
) -> ApiResult<ApiJson<DashboardOverviewResponse>> {
    Ok(ok(state.dashboard.overview(actor(current_user), query).await?))
}

pub async fn activity(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardActivityRequest>,
) -> ApiResult<ApiJson<DashboardActivityResponse>> {
    Ok(ok(state.dashboard.activity(actor(current_user), query).await?))
}

pub async fn filter_options(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardFilterOptionsRequest>,
) -> ApiResult<ApiJson<DashboardFilterOptionsResponse>> {
    Ok(ok(state.dashboard.filter_options(actor(current_user), query).await?))
}

pub async fn user_stats_leaderboard(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardUserStatsLeaderboardRequest>,
) -> ApiResult<ApiJson<DashboardUserStatsLeaderboardResponse>> {
    Ok(ok(state.dashboard.user_stats_leaderboard(actor(current_user), query).await?))
}

pub async fn user_usage_stats(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardUserUsageStatsRequest>,
) -> ApiResult<ApiJson<DashboardUserUsageStatsResponse>> {
    Ok(ok(state.dashboard.user_usage_stats(actor(current_user), query).await?))
}

pub async fn user_stats_time_series(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardUserStatsTimeSeriesRequest>,
) -> ApiResult<ApiJson<Vec<DashboardUserStatsTimeSeriesPoint>>> {
    Ok(ok(state.dashboard.user_stats_time_series(actor(current_user), query).await?))
}

pub async fn cost_forecast(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardCostForecastRequest>,
) -> ApiResult<ApiJson<DashboardCostForecastResponse>> {
    Ok(ok(state.dashboard.cost_forecast(actor(current_user), query).await?))
}

pub async fn cost_savings(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardCostSavingsRequest>,
) -> ApiResult<ApiJson<DashboardCostSavingsResponse>> {
    Ok(ok(state.dashboard.cost_savings(actor(current_user), query).await?))
}

pub async fn api_key_leaderboard(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardApiKeyLeaderboardRequest>,
) -> ApiResult<ApiJson<DashboardApiKeyLeaderboardResponse>> {
    Ok(ok(state.dashboard.api_key_leaderboard(actor(current_user), query).await?))
}

pub async fn provider_aggregation(
    State(state): State<DashboardApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DashboardProviderAggregationRequest>,
) -> ApiResult<ApiJson<Vec<DashboardProviderAggregationItem>>> {
    Ok(ok(state.dashboard.provider_aggregation(actor(current_user), query).await?))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

fn actor(current_user: CurrentUser) -> DashboardActor {
    DashboardActor {
        user_id: current_user.id,
        role: current_user.role,
    }
}
