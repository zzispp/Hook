use axum::{
    Extension, Json,
    extract::{Query, State},
};
use rbac::api::CurrentUser;
use types::{
    dashboard::{
        DashboardActivityRequest, DashboardActivityResponse, DashboardFilterOptionsRequest, DashboardFilterOptionsResponse, DashboardOverviewRequest,
        DashboardOverviewResponse,
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

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

fn actor(current_user: CurrentUser) -> DashboardActor {
    DashboardActor {
        user_id: current_user.id,
        role: current_user.role,
    }
}
