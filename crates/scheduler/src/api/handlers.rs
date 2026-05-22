use axum::{
    Json,
    extract::{Path, Query, State},
};
use types::{
    pagination::Page,
    response::ApiResponse,
    scheduler::{ScheduledTask, ScheduledTaskRun, ScheduledTaskRunListRequest, ScheduledTaskUpdate},
};

use crate::api::{SchedulerApiError, SchedulerApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, SchedulerApiError>;

pub async fn list_tasks(State(state): State<SchedulerApiState>) -> ApiResult<ApiJson<Vec<ScheduledTask>>> {
    Ok(ok(state.scheduler.list_tasks().await?))
}

pub async fn update_task(
    State(state): State<SchedulerApiState>,
    Path(code): Path<String>,
    Json(payload): Json<ScheduledTaskUpdate>,
) -> ApiResult<ApiJson<ScheduledTask>> {
    Ok(ok(state.scheduler.update_task(&code, payload).await?))
}

pub async fn list_runs(
    State(state): State<SchedulerApiState>,
    Query(query): Query<ScheduledTaskRunListRequest>,
) -> ApiResult<ApiJson<Page<ScheduledTaskRun>>> {
    Ok(ok(state.scheduler.list_runs(query).await?))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
