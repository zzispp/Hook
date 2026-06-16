use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
};
use storage::provider::ProviderStore;
use types::{
    provider::{RoutingProfile, RoutingProfileUpsert, RoutingProfilesResponse, RoutingRankingResponse, RoutingRankingsRequest},
    response::{ApiErrorResponse, ApiResponse},
};

use crate::llm_proxy::{LlmProxyError, LlmProxyState, routing, routing_rankings};

#[derive(Clone)]
pub struct RoutingApiState {
    llm_proxy: LlmProxyState,
}

#[derive(Debug)]
pub struct RoutingApiError(String);

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, RoutingApiError>;

impl RoutingApiState {
    pub fn new(llm_proxy: LlmProxyState) -> Self {
        Self { llm_proxy }
    }
}

pub fn create_router(state: RoutingApiState) -> Router {
    Router::new()
        .route("/admin/routing/profiles", get(profiles))
        .route("/admin/routing/profiles/{id}", put(upsert_profile))
        .route("/admin/routing/rankings", get(rankings))
        .route("/admin/routing/decisions/{request_id}", get(decision))
        .with_state(state)
}

async fn profiles(State(state): State<RoutingApiState>) -> ApiResult<ApiJson<RoutingProfilesResponse>> {
    let profiles = routing::list_profiles(&state.llm_proxy).await?;
    Ok(ok(RoutingProfilesResponse { profiles }))
}

async fn upsert_profile(
    State(state): State<RoutingApiState>,
    Path(id): Path<String>,
    Json(patch): Json<RoutingProfileUpsert>,
) -> ApiResult<ApiJson<RoutingProfile>> {
    let profile = routing::upsert_profile(&state.llm_proxy, routing::profile_id_from_str(&id), patch).await?;
    Ok(ok(profile))
}

async fn rankings(State(state): State<RoutingApiState>, Query(query): Query<RoutingRankingsRequest>) -> ApiResult<ApiJson<RoutingRankingResponse>> {
    Ok(ok(routing_rankings(&state.llm_proxy, query).await?))
}

async fn decision(State(state): State<RoutingApiState>, Path(request_id): Path<String>) -> ApiResult<ApiJson<types::provider::RoutingDecisionResponse>> {
    let store = ProviderStore::new(state.llm_proxy.database());
    let response = store
        .get_routing_decision_sample(&request_id)
        .await?
        .ok_or_else(|| RoutingApiError(format!("routing decision not found: {request_id}")))?;
    Ok(ok(response))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

impl From<LlmProxyError> for RoutingApiError {
    fn from(value: LlmProxyError) -> Self {
        Self(value.to_string())
    }
}

impl From<storage::StorageError> for RoutingApiError {
    fn from(value: storage::StorageError) -> Self {
        Self(value.to_string())
    }
}

impl IntoResponse for RoutingApiError {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(ApiErrorResponse::new(self.0))).into_response()
    }
}
