use axum::{
    Extension, Router,
    routing::{get, put},
};

use super::{
    CurrentUser,
    handlers::{
        api_menu_bindings, create_api, create_menu_item, create_menu_section, create_role, delete_api, delete_menu_item, delete_menu_section, delete_role,
        list_apis, list_menu_items, list_menu_sections, list_roles, list_unbound_apis, menu_api_bindings, navbar, replace_api, replace_api_menus,
        replace_menu_apis, replace_menu_item, replace_menu_section, replace_role, replace_role_permissions, role_permission_bindings,
    },
    state::RbacApiState,
};

pub fn create_router(state: RbacApiState) -> Router {
    Router::new()
        .route("/navbar", get(navbar_route))
        .route("/rbac/roles", get(list_roles).post(create_role))
        .route("/rbac/roles/{code}", put(replace_role).delete(delete_role))
        .route("/rbac/roles/{code}/permissions", get(role_permission_bindings).put(replace_role_permissions))
        .route("/rbac/apis", get(list_apis).post(create_api))
        .route("/rbac/apis/unbound", get(list_unbound_apis))
        .route("/rbac/apis/{id}", put(replace_api).delete(delete_api))
        .route("/rbac/apis/{id}/menus", get(api_menu_bindings).put(replace_api_menus))
        .route("/rbac/menu-sections", get(list_menu_sections).post(create_menu_section))
        .route("/rbac/menu-sections/{id}", put(replace_menu_section).delete(delete_menu_section))
        .route("/rbac/menu-items", get(list_menu_items).post(create_menu_item))
        .route("/rbac/menu-items/{id}", put(replace_menu_item).delete(delete_menu_item))
        .route("/rbac/menu-items/{id}/apis", get(menu_api_bindings).put(replace_menu_apis))
        .with_state(state)
}

async fn navbar_route(
    state: axum::extract::State<RbacApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<axum::Json<types::response::ApiResponse<types::rbac::NavResponse>>, super::RbacApiError> {
    navbar(state, current_user.role).await
}
