use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde_json::Value;
use types::{
    model::{
        BatchDeleteGlobalModelsRequest, BatchDeleteGlobalModelsResponse, GlobalModelCreate, GlobalModelListRequest, GlobalModelListResponse,
        GlobalModelProvidersResponse, GlobalModelResponse, GlobalModelUpdate, GlobalModelWithStats, ModelCatalogResponse,
    },
    response::ApiResponse,
};

use crate::api::{ModelApiState, error::ModelApiError};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, ModelApiError>;

pub async fn list_global_models(
    State(state): State<ModelApiState>,
    Query(query): Query<GlobalModelListRequest>,
) -> ApiResult<ApiJson<GlobalModelListResponse>> {
    Ok(ok(state.models.list_global_models(query).await?))
}

pub async fn get_global_model(State(state): State<ModelApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<GlobalModelWithStats>> {
    Ok(ok(state.models.get_global_model(&id).await?))
}

pub async fn create_global_model(State(state): State<ModelApiState>, Json(payload): Json<GlobalModelCreate>) -> ApiResult<ApiJson<GlobalModelResponse>> {
    Ok(ok(state.models.create_global_model(payload).await?))
}

pub async fn update_global_model(
    State(state): State<ModelApiState>,
    Path(id): Path<String>,
    Json(payload): Json<GlobalModelUpdate>,
) -> ApiResult<ApiJson<GlobalModelResponse>> {
    Ok(ok(state.models.update_global_model(&id, payload).await?))
}

pub async fn delete_global_model(State(state): State<ModelApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.models.delete_global_model(&id).await?;
    Ok(ok(()))
}

pub async fn batch_delete_global_models(
    State(state): State<ModelApiState>,
    Json(payload): Json<BatchDeleteGlobalModelsRequest>,
) -> ApiResult<ApiJson<BatchDeleteGlobalModelsResponse>> {
    Ok(ok(state.models.batch_delete_global_models(payload.ids).await?))
}

pub async fn global_model_providers(State(state): State<ModelApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<GlobalModelProvidersResponse>> {
    Ok(ok(state.models.global_model_providers(&id).await?))
}

pub async fn catalog(State(state): State<ModelApiState>) -> ApiResult<ApiJson<ModelCatalogResponse>> {
    Ok(ok(state.models.catalog().await?))
}

pub async fn public_catalog(State(state): State<ModelApiState>, Query(query): Query<GlobalModelListRequest>) -> ApiResult<ApiJson<GlobalModelListResponse>> {
    Ok(ok(state
        .models
        .list_global_models(GlobalModelListRequest {
            is_active: Some(true),
            ..query
        })
        .await?))
}

pub async fn external_models(State(state): State<ModelApiState>) -> ApiResult<ApiJson<Value>> {
    Ok(ok(state.models.external_models().await?))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
