use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use types::{
    i18n::{
        I18nResourceResponse, TranslationBundleResponse, TranslationBundleUpsert, TranslationEntryCreate, TranslationEntryListRequest,
        TranslationEntryListResponse, TranslationEntryResponse, TranslationEntryUpdate, TranslationLanguageCreate, TranslationLanguageListRequest,
        TranslationLanguageListResponse, TranslationLanguageResponse, TranslationLanguageUpdate,
    },
    response::ApiResponse,
};

use crate::api::{I18nApiError, I18nApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, I18nApiError>;

#[derive(Debug, Deserialize)]
pub struct ResourceQuery {
    pub lang: String,
    pub namespace: String,
}

pub async fn resource_bundle(State(state): State<I18nApiState>, Query(query): Query<ResourceQuery>) -> ApiResult<ApiJson<I18nResourceResponse>> {
    Ok(ok(state.i18n.resource_bundle(&query.lang, &query.namespace).await?))
}

pub async fn list_languages(
    State(state): State<I18nApiState>,
    Query(query): Query<TranslationLanguageListRequest>,
) -> ApiResult<ApiJson<TranslationLanguageListResponse>> {
    Ok(ok(state.i18n.list_languages(query).await?))
}

pub async fn create_language(
    State(state): State<I18nApiState>,
    Json(payload): Json<TranslationLanguageCreate>,
) -> ApiResult<ApiJson<TranslationLanguageResponse>> {
    Ok(ok(state.i18n.create_language(payload).await?))
}

pub async fn update_language(
    State(state): State<I18nApiState>,
    Path(code): Path<String>,
    Json(payload): Json<TranslationLanguageUpdate>,
) -> ApiResult<ApiJson<TranslationLanguageResponse>> {
    Ok(ok(state.i18n.update_language(&code, payload).await?))
}

pub async fn delete_language(State(state): State<I18nApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<()>> {
    state.i18n.delete_language(&code).await?;
    Ok(ok(()))
}

pub async fn list_entries(
    State(state): State<I18nApiState>,
    Query(query): Query<TranslationEntryListRequest>,
) -> ApiResult<ApiJson<TranslationEntryListResponse>> {
    Ok(ok(state.i18n.list_entries(query).await?))
}

pub async fn create_entry(State(state): State<I18nApiState>, Json(payload): Json<TranslationEntryCreate>) -> ApiResult<ApiJson<TranslationEntryResponse>> {
    Ok(ok(state.i18n.create_entry(payload).await?))
}

pub async fn update_entry(
    State(state): State<I18nApiState>,
    Path(id): Path<String>,
    Json(payload): Json<TranslationEntryUpdate>,
) -> ApiResult<ApiJson<TranslationEntryResponse>> {
    Ok(ok(state.i18n.update_entry(&id, payload).await?))
}

pub async fn delete_entry(State(state): State<I18nApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.i18n.delete_entry(&id).await?;
    Ok(ok(()))
}

pub async fn upsert_bundle(
    State(state): State<I18nApiState>,
    Path((namespace, group_key, item_key)): Path<(String, String, String)>,
    Json(payload): Json<TranslationBundleUpsert>,
) -> ApiResult<ApiJson<TranslationBundleResponse>> {
    Ok(ok(state.i18n.upsert_bundle(&namespace, &group_key, &item_key, payload).await?))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
