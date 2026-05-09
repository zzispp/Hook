use async_trait::async_trait;
use types::{
    pagination::Page,
    rbac::{
        ApiMenuBindingInput, ApiPermission, ApiPermissionInput, MenuApiBindingInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse,
        PermissionSnapshot, RbacListRequest, Role, RoleInput, RolePermissionBindingInput,
    },
};

use super::RbacResult;

/// Persists RBAC roles, API permissions, menus, and bindings.
#[async_trait]
pub trait RbacRepository: Send + Sync + 'static {
    async fn create_role(&self, input: types::rbac::RoleInput) -> RbacResult<types::rbac::Role>;
    async fn create_system_role(&self, input: types::rbac::RoleInput) -> RbacResult<types::rbac::Role>;
    async fn replace_role(&self, code: &str, input: types::rbac::RoleInput) -> RbacResult<types::rbac::Role>;
    async fn replace_system_role(&self, code: &str, input: types::rbac::RoleInput) -> RbacResult<types::rbac::Role>;
    async fn delete_role(&self, code: &str) -> RbacResult<()>;
    async fn find_role(&self, code: &str) -> RbacResult<Option<types::rbac::Role>>;
    async fn role_has_menu_bindings(&self, code: &str) -> RbacResult<bool>;
    async fn role_has_api_bindings(&self, code: &str) -> RbacResult<bool>;
    async fn role_has_users(&self, code: &str) -> RbacResult<bool>;
    async fn list_roles(&self) -> RbacResult<Vec<types::rbac::Role>>;
    async fn page_roles(&self, request: RbacListRequest) -> RbacResult<Page<Role>>;
    async fn create_api(&self, input: types::rbac::ApiPermissionInput) -> RbacResult<types::rbac::ApiPermission>;
    async fn replace_api(&self, id: &str, input: types::rbac::ApiPermissionInput) -> RbacResult<types::rbac::ApiPermission>;
    async fn delete_api(&self, id: &str) -> RbacResult<()>;
    async fn find_api(&self, id: &str) -> RbacResult<Option<types::rbac::ApiPermission>>;
    async fn api_has_menu_bindings(&self, id: &str) -> RbacResult<bool>;
    async fn api_has_role_bindings(&self, id: &str) -> RbacResult<bool>;
    async fn list_apis(&self) -> RbacResult<Vec<types::rbac::ApiPermission>>;
    async fn page_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>>;
    async fn page_unbound_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>>;
    async fn create_menu_section(&self, input: types::rbac::MenuSectionInput) -> RbacResult<types::rbac::MenuSection>;
    async fn replace_menu_section(&self, id: &str, input: types::rbac::MenuSectionInput) -> RbacResult<types::rbac::MenuSection>;
    async fn delete_menu_section(&self, id: &str) -> RbacResult<()>;
    async fn find_menu_section(&self, id: &str) -> RbacResult<Option<types::rbac::MenuSection>>;
    async fn menu_section_has_items(&self, id: &str) -> RbacResult<bool>;
    async fn page_menu_sections(&self, request: RbacListRequest) -> RbacResult<Page<MenuSection>>;
    async fn create_menu_item(&self, input: types::rbac::MenuItemInput) -> RbacResult<types::rbac::MenuItem>;
    async fn replace_menu_item(&self, id: &str, input: types::rbac::MenuItemInput) -> RbacResult<types::rbac::MenuItem>;
    async fn delete_menu_item(&self, id: &str) -> RbacResult<()>;
    async fn find_menu_item(&self, id: &str) -> RbacResult<Option<types::rbac::MenuItem>>;
    async fn menu_item_has_children(&self, id: &str) -> RbacResult<bool>;
    async fn menu_item_has_role_bindings(&self, id: &str) -> RbacResult<bool>;
    async fn menu_item_has_api_bindings(&self, id: &str) -> RbacResult<bool>;
    async fn list_menu_items(&self) -> RbacResult<Vec<types::rbac::MenuItem>>;
    async fn page_menu_items(&self, request: RbacListRequest) -> RbacResult<Page<MenuItem>>;
    async fn replace_menu_apis(&self, menu_item_id: &str, input: MenuApiBindingInput) -> RbacResult<()>;
    async fn replace_api_menus(&self, api_permission_id: &str, input: ApiMenuBindingInput) -> RbacResult<()>;
    async fn replace_role_permissions(&self, role_code: &str, input: RolePermissionBindingInput) -> RbacResult<()>;
    async fn menu_api_ids(&self, menu_item_id: &str) -> RbacResult<Vec<String>>;
    async fn api_menu_ids(&self, api_permission_id: &str) -> RbacResult<Vec<String>>;
    async fn role_menu_item_ids(&self, role_code: &str) -> RbacResult<Vec<String>>;
    async fn role_api_ids(&self, role_code: &str) -> RbacResult<Vec<String>>;
    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot>;
}

/// Stores and reads RBAC cache snapshots. Missing cache data is an explicit infrastructure error.
#[async_trait]
pub trait RbacCache: Send + Sync + 'static {
    async fn write_snapshot(&self, snapshot: &PermissionSnapshot) -> RbacResult<()>;
    async fn read_snapshot(&self) -> RbacResult<PermissionSnapshot>;
    async fn read_nav(&self, role_code: &str) -> RbacResult<NavResponse>;
}

#[async_trait]
pub trait RbacUseCase: Send + Sync + 'static {
    async fn navbar(&self, role_code: &str) -> RbacResult<NavResponse>;
    async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()>;
    fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool>;
}

#[async_trait]
pub trait RbacAdminUseCase: Send + Sync + 'static {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role>;
    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role>;
    async fn delete_role(&self, code: &str) -> RbacResult<()>;
    async fn page_roles(&self, request: RbacListRequest) -> RbacResult<Page<Role>>;
    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission>;
    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission>;
    async fn delete_api(&self, id: &str) -> RbacResult<()>;
    async fn page_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>>;
    async fn page_unbound_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>>;
    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection>;
    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection>;
    async fn delete_menu_section(&self, id: &str) -> RbacResult<()>;
    async fn page_menu_sections(&self, request: RbacListRequest) -> RbacResult<Page<MenuSection>>;
    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem>;
    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem>;
    async fn delete_menu_item(&self, id: &str) -> RbacResult<()>;
    async fn page_menu_items(&self, request: RbacListRequest) -> RbacResult<Page<MenuItem>>;
    async fn replace_menu_apis(&self, menu_item_id: &str, input: MenuApiBindingInput) -> RbacResult<()>;
    async fn replace_api_menus(&self, api_permission_id: &str, input: ApiMenuBindingInput) -> RbacResult<()>;
    async fn replace_role_permissions(&self, role_code: &str, input: RolePermissionBindingInput) -> RbacResult<()>;
    async fn menu_api_ids(&self, menu_item_id: &str) -> RbacResult<Vec<String>>;
    async fn api_menu_ids(&self, api_permission_id: &str) -> RbacResult<Vec<String>>;
    async fn role_permission_bindings(&self, role_code: &str) -> RbacResult<RolePermissionBindingInput>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiCheckRequest {
    pub method: String,
    pub path: String,
    pub role_code: String,
    pub system: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthWhitelistRule {
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationConfig {
    pub whitelist: Vec<AuthWhitelistRule>,
    pub authenticated: Vec<AuthWhitelistRule>,
}
