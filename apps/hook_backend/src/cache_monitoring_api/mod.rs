use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use storage::{
    Database, StorageError,
    api_token::api_token_records,
    model::global_models,
    provider::record::{provider_api_keys, provider_endpoints, providers},
    user::UserStore,
};
use time::OffsetDateTime;
use types::{
    api_token::ApiTokenOwnerResponse,
    cache_monitoring::{CacheAffinityItem, CacheAffinityListRequest},
    pagination::Page,
    response::{ApiErrorResponse, ApiResponse},
    user::User,
};

use crate::llm_proxy::{AffinityEntry, ClearAffinityInput, LlmProxyError, LlmProxyState};

#[cfg(test)]
mod tests;

const MAX_PAGE_SIZE: u64 = 100;

#[derive(Clone)]
pub struct CacheMonitoringApiState {
    database: Database,
    llm_proxy: LlmProxyState,
    system_owner: Option<(String, ApiTokenOwnerResponse)>,
}

#[derive(Debug)]
pub enum CacheMonitoringApiError {
    InvalidInput(String),
    NotFound(String),
    ServiceUnavailable(String),
    Internal(String),
}

#[derive(Default)]
struct AffinityLookups {
    tokens: HashMap<String, api_token_records::Model>,
    users: HashMap<String, User>,
    providers: HashMap<String, providers::Model>,
    endpoints: HashMap<String, provider_endpoints::Model>,
    keys: HashMap<String, provider_api_keys::Model>,
    models: HashMap<String, global_models::Model>,
}

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, CacheMonitoringApiError>;

impl CacheMonitoringApiState {
    pub fn new(database: Database, llm_proxy: LlmProxyState, system_owner: Option<(String, ApiTokenOwnerResponse)>) -> Self {
        Self {
            database,
            llm_proxy,
            system_owner,
        }
    }
}

pub fn create_router(state: CacheMonitoringApiState) -> Router {
    Router::new()
        .route("/admin/monitoring/cache/affinities", get(list_affinities))
        .route(
            "/admin/monitoring/cache/affinities/{affinity_key}/{endpoint_id}/{model_id}/{api_format}",
            delete(delete_affinity),
        )
        .route("/admin/monitoring/cache", delete(clear_affinities))
        .with_state(state)
}

async fn list_affinities(
    State(state): State<CacheMonitoringApiState>,
    Query(query): Query<CacheAffinityListRequest>,
) -> ApiResult<ApiJson<Page<CacheAffinityItem>>> {
    let (page, page_size) = validated_page(query.page, query.page_size)?;
    let entries = state.llm_proxy.list_affinities().await?;
    let lookups = load_lookups(&state.database, &entries).await?;
    let items = filter_items(
        build_items(entries, &lookups, OffsetDateTime::now_utc().unix_timestamp(), state.system_owner.as_ref()),
        normalized_search(query.search.as_deref()),
    );
    Ok(ok(paginate(items, page, page_size)))
}

async fn delete_affinity(
    State(state): State<CacheMonitoringApiState>,
    Path((affinity_key, endpoint_id, model_id, api_format)): Path<(String, String, String, String)>,
) -> ApiResult<ApiJson<()>> {
    let deleted = state
        .llm_proxy
        .clear_single_affinity(ClearAffinityInput {
            token_id: &affinity_key,
            model_id: &model_id,
            api_format: &api_format,
            endpoint_id: &endpoint_id,
        })
        .await?;
    if !deleted {
        return Err(CacheMonitoringApiError::NotFound("cache affinity not found".into()));
    }
    Ok(ok(()))
}

async fn clear_affinities(State(state): State<CacheMonitoringApiState>) -> ApiResult<ApiJson<()>> {
    state.llm_proxy.clear_all_affinities().await?;
    Ok(ok(()))
}

async fn load_lookups(database: &Database, entries: &[AffinityEntry]) -> ApiResult<AffinityLookups> {
    let db = database.connection();
    let token_ids: HashSet<_> = entries.iter().map(|entry| entry.token_id.clone()).collect();
    let provider_ids: HashSet<_> = entries.iter().map(|entry| entry.record.provider_id.clone()).collect();
    let endpoint_ids: HashSet<_> = entries.iter().map(|entry| entry.record.endpoint_id.clone()).collect();
    let key_ids: HashSet<_> = entries.iter().map(|entry| entry.record.key_id.clone()).collect();
    let model_ids: HashSet<_> = entries.iter().map(|entry| entry.record.model_id.clone()).collect();

    let token_rows = find_tokens(db, token_ids).await?;
    let user_ids: HashSet<_> = token_rows.iter().filter_map(|record| record.user_id.clone()).collect();

    Ok(AffinityLookups {
        tokens: token_rows.into_iter().map(|record| (record.id.clone(), record)).collect(),
        users: find_users(database, user_ids)
            .await?
            .into_iter()
            .map(|user| (user.id.0.clone(), user))
            .collect(),
        providers: find_providers(db, provider_ids)
            .await?
            .into_iter()
            .map(|record| (record.id.clone(), record))
            .collect(),
        endpoints: find_endpoints(db, endpoint_ids)
            .await?
            .into_iter()
            .map(|record| (record.id.clone(), record))
            .collect(),
        keys: find_keys(db, key_ids).await?.into_iter().map(|record| (record.id.clone(), record)).collect(),
        models: find_models(db, model_ids)
            .await?
            .into_iter()
            .map(|record| (record.id.clone(), record))
            .collect(),
    })
}

fn build_items(
    entries: Vec<AffinityEntry>,
    lookups: &AffinityLookups,
    now: i64,
    system_owner: Option<&(String, ApiTokenOwnerResponse)>,
) -> Vec<CacheAffinityItem> {
    let mut items: Vec<_> = entries
        .into_iter()
        .map(|entry| {
            let token = lookups.tokens.get(&entry.token_id);
            let user = token.and_then(|record| resolve_owner(record.user_id.as_deref(), &lookups.users, system_owner));
            let provider = lookups.providers.get(&entry.record.provider_id);
            let endpoint = lookups.endpoints.get(&entry.record.endpoint_id);
            let key = lookups.keys.get(&entry.record.key_id);
            let model = lookups.models.get(&entry.record.model_id);
            CacheAffinityItem {
                affinity_key: entry.token_id,
                user_id: token.and_then(|record| record.user_id.clone()),
                username: user.as_ref().map(|record| record.username.clone()),
                user_email: user.as_ref().map(|record| record.email.clone()),
                token_name: token.map(|record| record.name.clone()),
                token_prefix: token.map(|record| record.token_prefix.clone()),
                provider_id: entry.record.provider_id.clone(),
                provider_name: provider.map(|record| record.name.clone()),
                endpoint_id: entry.record.endpoint_id.clone(),
                endpoint_base_url: endpoint.map(|record| record.base_url.clone()),
                endpoint_api_format: endpoint.map(|record| record.api_format.clone()),
                provider_key_id: entry.record.key_id.clone(),
                provider_key_name: key.map(|record| record.name.clone()),
                model_id: entry.record.model_id.clone(),
                model_name: model.map(model_name),
                api_format: entry.record.api_format.clone(),
                ttl_seconds: (entry.record.expire_at - now).max(0),
                request_count: entry.record.request_count,
            }
        })
        .collect();
    items.sort_by(|left, right| {
        right
            .ttl_seconds
            .cmp(&left.ttl_seconds)
            .then_with(|| right.request_count.cmp(&left.request_count))
    });
    items
}

fn filter_items(items: Vec<CacheAffinityItem>, search: Option<String>) -> Vec<CacheAffinityItem> {
    let Some(search) = search else { return items };
    items.into_iter().filter(|item| item_matches_search(item, &search)).collect()
}

fn item_matches_search(item: &CacheAffinityItem, search: &str) -> bool {
    searchable_fields(item)
        .into_iter()
        .flatten()
        .any(|value| value.to_ascii_lowercase().contains(search))
}

fn searchable_fields(item: &CacheAffinityItem) -> [Option<&str>; 14] {
    [
        Some(item.affinity_key.as_str()),
        item.user_id.as_deref(),
        item.username.as_deref(),
        item.user_email.as_deref(),
        item.token_name.as_deref(),
        item.token_prefix.as_deref(),
        Some(item.provider_id.as_str()),
        item.provider_name.as_deref(),
        Some(item.endpoint_id.as_str()),
        item.endpoint_api_format.as_deref(),
        item.provider_key_name.as_deref(),
        Some(item.model_id.as_str()),
        item.model_name.as_deref(),
        Some(item.api_format.as_str()),
    ]
}

fn paginate(items: Vec<CacheAffinityItem>, page: u64, page_size: u64) -> Page<CacheAffinityItem> {
    let total = items.len() as u64;
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(items.len());
    let page_items = if start >= items.len() { Vec::new() } else { items[start..end].to_vec() };
    Page {
        items: page_items,
        total,
        page,
        page_size,
    }
}

fn validated_page(page: u64, page_size: u64) -> ApiResult<(u64, u64)> {
    if page == 0 {
        return Err(CacheMonitoringApiError::InvalidInput("page must be greater than 0".into()));
    }
    if page_size == 0 || page_size > MAX_PAGE_SIZE {
        return Err(CacheMonitoringApiError::InvalidInput(format!(
            "page_size must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    Ok((page, page_size))
}

fn normalized_search(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(|value| value.to_ascii_lowercase())
}

fn model_name(record: &global_models::Model) -> String {
    if record.display_name.trim().is_empty() {
        record.name.clone()
    } else {
        record.display_name.clone()
    }
}

fn resolve_owner(
    user_id: Option<&str>,
    users: &HashMap<String, User>,
    system_owner: Option<&(String, ApiTokenOwnerResponse)>,
) -> Option<ApiTokenOwnerResponse> {
    let user_id = user_id?;
    if let Some((owner_id, owner)) = system_owner
        && owner_id == user_id
    {
        return Some(owner.clone());
    }
    users.get(user_id).map(|record| ApiTokenOwnerResponse {
        username: record.username.clone(),
        email: record.email.clone(),
        group_codes: record.group_codes.clone(),
    })
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

async fn find_tokens(db: &sea_orm::DatabaseConnection, ids: HashSet<String>) -> Result<Vec<api_token_records::Model>, sea_orm::DbErr> {
    find_by_ids(ids, api_token_records::Entity::find, api_token_records::Column::Id, db).await
}
async fn find_users(database: &Database, ids: HashSet<String>) -> Result<Vec<User>, StorageError> {
    UserStore::new(database.clone()).find_by_ids(&ids.into_iter().collect::<Vec<_>>()).await
}
async fn find_providers(db: &sea_orm::DatabaseConnection, ids: HashSet<String>) -> Result<Vec<providers::Model>, sea_orm::DbErr> {
    find_by_ids(ids, providers::Entity::find, providers::Column::Id, db).await
}
async fn find_endpoints(db: &sea_orm::DatabaseConnection, ids: HashSet<String>) -> Result<Vec<provider_endpoints::Model>, sea_orm::DbErr> {
    find_by_ids(ids, provider_endpoints::Entity::find, provider_endpoints::Column::Id, db).await
}
async fn find_keys(db: &sea_orm::DatabaseConnection, ids: HashSet<String>) -> Result<Vec<provider_api_keys::Model>, sea_orm::DbErr> {
    find_by_ids(ids, provider_api_keys::Entity::find, provider_api_keys::Column::Id, db).await
}
async fn find_models(db: &sea_orm::DatabaseConnection, ids: HashSet<String>) -> Result<Vec<global_models::Model>, sea_orm::DbErr> {
    find_by_ids(ids, global_models::Entity::find, global_models::Column::Id, db).await
}

async fn find_by_ids<E, C, S>(ids: HashSet<String>, select: S, column: C, db: &sea_orm::DatabaseConnection) -> Result<Vec<E::Model>, sea_orm::DbErr>
where
    E: EntityTrait,
    C: ColumnTrait,
    S: FnOnce() -> sea_orm::Select<E>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    select().filter(column.is_in(ids)).all(db).await
}

impl From<sea_orm::DbErr> for CacheMonitoringApiError {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<StorageError> for CacheMonitoringApiError {
    fn from(value: StorageError) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<LlmProxyError> for CacheMonitoringApiError {
    fn from(value: LlmProxyError) -> Self {
        match value {
            LlmProxyError::Infrastructure(message) => Self::ServiceUnavailable(message),
            LlmProxyError::InvalidRequest(message) => Self::InvalidInput(message),
            LlmProxyError::NotFound(message) => Self::NotFound(message),
            other => Self::Internal(other.to_string()),
        }
    }
}

impl IntoResponse for CacheMonitoringApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidInput(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Self::ServiceUnavailable(message) => (StatusCode::SERVICE_UNAVAILABLE, message),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (status, Json(ApiErrorResponse::new(message))).into_response()
    }
}
