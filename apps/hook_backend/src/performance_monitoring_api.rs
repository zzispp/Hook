use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use std::sync::Arc;
use storage::{Database, performance_monitoring::PerformanceMonitoringStore};
use types::{
    performance_monitoring::{
        HostRealtimeMetrics, PerformanceMonitoringOverviewRequest, PerformanceMonitoringOverviewResponse, PerformanceMonitoringRealtimeResponse,
    },
    response::{ApiErrorResponse, ApiResponse},
};

use crate::performance_monitoring_os::PerformanceOsCollector;

const REALTIME_WINDOW_MINUTES: i64 = 5;

#[derive(Clone)]
pub struct PerformanceMonitoringApiState {
    database: Database,
    os_collector: Arc<PerformanceOsCollector>,
}

#[derive(Debug)]
pub struct PerformanceMonitoringApiError(String);

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, PerformanceMonitoringApiError>;

impl PerformanceMonitoringApiState {
    pub fn new(database: Database, os_collector: Arc<PerformanceOsCollector>) -> Self {
        Self { database, os_collector }
    }
}

pub fn create_router(state: PerformanceMonitoringApiState) -> Router {
    Router::new()
        .route("/admin/performance-monitoring/overview", get(overview))
        .route("/admin/performance-monitoring/realtime", get(realtime))
        .with_state(state)
}

async fn overview(
    State(state): State<PerformanceMonitoringApiState>,
    Query(query): Query<PerformanceMonitoringOverviewRequest>,
) -> ApiResult<ApiJson<PerformanceMonitoringOverviewResponse>> {
    let system = state.os_collector.clone().snapshot().await.map_err(PerformanceMonitoringApiError::from)?;
    let response = PerformanceMonitoringStore::new(state.database)
        .overview_with_system(query.range, time::OffsetDateTime::now_utc(), system)
        .await?;
    Ok(ok(response))
}

async fn realtime(State(state): State<PerformanceMonitoringApiState>) -> ApiResult<ApiJson<PerformanceMonitoringRealtimeResponse>> {
    let store = PerformanceMonitoringStore::new(state.database);
    let system = state.os_collector.clone().snapshot().await.map_err(PerformanceMonitoringApiError::from)?;
    let snapshot = store.aggregate_point(realtime_window(time::OffsetDateTime::now_utc()), system.clone()).await?;
    Ok(ok(PerformanceMonitoringRealtimeResponse {
        snapshot: Some(snapshot),
        host: HostRealtimeMetrics {
            collected_at: format_timestamp(time::OffsetDateTime::now_utc()),
            metrics: system.host,
        },
    }))
}

fn realtime_window(now: time::OffsetDateTime) -> storage::performance_monitoring::SnapshotAggregationWindow {
    storage::performance_monitoring::SnapshotAggregationWindow {
        granularity: types::performance_monitoring::SnapshotGranularity::Minute,
        started_at: now - time::Duration::minutes(REALTIME_WINDOW_MINUTES),
        ended_at: now,
    }
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

fn format_timestamp(value: time::OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .expect("performance realtime timestamp must format as RFC3339")
}

impl From<storage::StorageError> for PerformanceMonitoringApiError {
    fn from(value: storage::StorageError) -> Self {
        Self(value.to_string())
    }
}

impl From<crate::performance_monitoring_os::PerformanceOsCollectorError> for PerformanceMonitoringApiError {
    fn from(value: crate::performance_monitoring_os::PerformanceOsCollectorError) -> Self {
        Self(value.to_string())
    }
}

impl IntoResponse for PerformanceMonitoringApiError {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(ApiErrorResponse::new(self.0))).into_response()
    }
}
