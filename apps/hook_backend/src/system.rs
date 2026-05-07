use axum::{Json, Router, routing::get};
use serde::Serialize;
use types::response::ApiResponse;

type ApiJson<T> = Json<ApiResponse<T>>;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn create_router() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> ApiJson<HealthResponse> {
    Json(ApiResponse::new(HealthResponse { status: "ok" }))
}
