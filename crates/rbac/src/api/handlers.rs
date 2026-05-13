use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use types::{
    pagination::{Page, PageRequest},
    rbac::{
        ApiMenuBindingInput, ApiPermission, ApiPermissionInput, MenuApiBindingInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse,
        RbacListFilters, RbacListRequest, Role, RoleInput, RolePermissionBinding, RolePermissionBindingInput,
    },
    response::ApiResponse,
};

use crate::api::{RbacApiError, RbacApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, RbacApiError>;

#[derive(Debug, Deserialize)]
pub struct RbacListQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

pub async fn navbar(State(state): State<RbacApiState>, role: String) -> ApiResult<ApiJson<NavResponse>> {
    let nav = state.rbac.navbar(&role).await?;
    Ok(ok(nav))
}

pub async fn list_roles(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<Role>>> {
    Ok(ok(state.rbac_admin.page_roles(query.into()).await?))
}

pub async fn create_role(State(state): State<RbacApiState>, Json(payload): Json<RoleInput>) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.create_role(payload).await?))
}

pub async fn replace_role(State(state): State<RbacApiState>, Path(code): Path<String>, Json(payload): Json<RoleInput>) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.replace_role(&code, payload).await?))
}

pub async fn delete_role(State(state): State<RbacApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_role(&code).await?;
    Ok(ok(()))
}

pub async fn list_apis(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<ApiPermission>>> {
    Ok(ok(state.rbac_admin.page_apis(query.into()).await?))
}

pub async fn list_unbound_apis(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<ApiPermission>>> {
    Ok(ok(state.rbac_admin.page_unbound_apis(query.into()).await?))
}

pub async fn create_api(State(state): State<RbacApiState>, Json(payload): Json<ApiPermissionInput>) -> ApiResult<ApiJson<ApiPermission>> {
    Ok(ok(state.rbac_admin.create_api(payload).await?))
}

pub async fn replace_api(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    Json(payload): Json<ApiPermissionInput>,
) -> ApiResult<ApiJson<ApiPermission>> {
    Ok(ok(state.rbac_admin.replace_api(&id, payload).await?))
}

pub async fn delete_api(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_api(&id).await?;
    Ok(ok(()))
}

pub async fn list_menu_sections(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<MenuSection>>> {
    Ok(ok(state.rbac_admin.page_menu_sections(query.into()).await?))
}

pub async fn create_menu_section(State(state): State<RbacApiState>, Json(payload): Json<MenuSectionInput>) -> ApiResult<ApiJson<MenuSection>> {
    Ok(ok(state.rbac_admin.create_menu_section(payload).await?))
}

pub async fn replace_menu_section(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    Json(payload): Json<MenuSectionInput>,
) -> ApiResult<ApiJson<MenuSection>> {
    Ok(ok(state.rbac_admin.replace_menu_section(&id, payload).await?))
}

pub async fn delete_menu_section(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_menu_section(&id).await?;
    Ok(ok(()))
}

pub async fn list_menu_items(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<MenuItem>>> {
    Ok(ok(state.rbac_admin.page_menu_items(query.into()).await?))
}

pub async fn create_menu_item(State(state): State<RbacApiState>, Json(payload): Json<MenuItemInput>) -> ApiResult<ApiJson<MenuItem>> {
    Ok(ok(state.rbac_admin.create_menu_item(payload).await?))
}

pub async fn replace_menu_item(State(state): State<RbacApiState>, Path(id): Path<String>, Json(payload): Json<MenuItemInput>) -> ApiResult<ApiJson<MenuItem>> {
    Ok(ok(state.rbac_admin.replace_menu_item(&id, payload).await?))
}

pub async fn delete_menu_item(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_menu_item(&id).await?;
    Ok(ok(()))
}

pub async fn replace_menu_apis(State(state): State<RbacApiState>, Path(id): Path<String>, Json(payload): Json<MenuApiBindingInput>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_menu_apis(&id, payload).await?;
    Ok(ok(()))
}

pub async fn menu_api_bindings(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<MenuApiBindingInput>> {
    let api_permission_ids = state.rbac_admin.menu_api_ids(&id).await?;
    Ok(ok(MenuApiBindingInput { api_permission_ids }))
}

pub async fn replace_api_menus(State(state): State<RbacApiState>, Path(id): Path<String>, Json(payload): Json<ApiMenuBindingInput>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_api_menus(&id, payload).await?;
    Ok(ok(()))
}

pub async fn api_menu_bindings(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ApiMenuBindingInput>> {
    let menu_item_ids = state.rbac_admin.api_menu_ids(&id).await?;
    Ok(ok(ApiMenuBindingInput { menu_item_ids }))
}

pub async fn replace_role_permissions(
    State(state): State<RbacApiState>,
    Path(code): Path<String>,
    Json(payload): Json<RolePermissionBindingInput>,
) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_role_permissions(&code, payload).await?;
    Ok(ok(()))
}

pub async fn role_permission_bindings(State(state): State<RbacApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<RolePermissionBinding>> {
    Ok(ok(state.rbac_admin.role_permission_bindings(&code, &state.authorization).await?))
}

impl From<RbacListQuery> for RbacListRequest {
    fn from(value: RbacListQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            filters: RbacListFilters {
                search: value.search,
                enabled: value.enabled,
            },
        }
    }
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
