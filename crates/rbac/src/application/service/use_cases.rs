use async_trait::async_trait;
use types::{
    pagination::Page,
    rbac::{
        ApiMenuBindingInput, ApiPermission, ApiPermissionInput, MenuApiBindingInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse,
        RbacListRequest, Role, RoleInput, RolePermissionBindingInput,
    },
};

use crate::application::{ApiCheckRequest, AuthorizationConfig, RbacAdminUseCase, RbacCache, RbacRepository, RbacResult, RbacService, RbacUseCase};

#[async_trait]
impl<R, C> RbacUseCase for RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    async fn navbar(&self, role_code: &str) -> RbacResult<NavResponse> {
        self.navbar(role_code).await
    }

    async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()> {
        self.authorize_api(config, request).await
    }

    fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
        self.is_whitelisted(config, method, path)
    }
}

#[async_trait]
impl<R, C> RbacAdminUseCase for RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        self.create_role(input).await
    }

    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        self.replace_role(code, input).await
    }

    async fn delete_role(&self, code: &str) -> RbacResult<()> {
        self.delete_role(code).await
    }

    async fn page_roles(&self, request: RbacListRequest) -> RbacResult<Page<Role>> {
        self.page_roles(request).await
    }

    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        self.create_api(input).await
    }

    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        self.replace_api(id, input).await
    }

    async fn delete_api(&self, id: &str) -> RbacResult<()> {
        self.delete_api(id).await
    }

    async fn page_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>> {
        self.page_apis(request).await
    }

    async fn page_unbound_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>> {
        self.page_unbound_apis(request).await
    }

    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.create_menu_section(input).await
    }

    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.replace_menu_section(id, input).await
    }

    async fn delete_menu_section(&self, id: &str) -> RbacResult<()> {
        self.delete_menu_section(id).await
    }

    async fn page_menu_sections(&self, request: RbacListRequest) -> RbacResult<Page<MenuSection>> {
        self.page_menu_sections(request).await
    }

    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.create_menu_item(input).await
    }

    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.replace_menu_item(id, input).await
    }

    async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        self.delete_menu_item(id).await
    }

    async fn page_menu_items(&self, request: RbacListRequest) -> RbacResult<Page<MenuItem>> {
        self.page_menu_items(request).await
    }

    async fn replace_menu_apis(&self, menu_item_id: &str, input: MenuApiBindingInput) -> RbacResult<()> {
        self.replace_menu_apis(menu_item_id, input).await
    }

    async fn replace_api_menus(&self, api_permission_id: &str, input: ApiMenuBindingInput) -> RbacResult<()> {
        self.replace_api_menus(api_permission_id, input).await
    }

    async fn replace_role_permissions(&self, role_code: &str, input: RolePermissionBindingInput) -> RbacResult<()> {
        self.replace_role_permissions(role_code, input).await
    }

    async fn menu_api_ids(&self, menu_item_id: &str) -> RbacResult<Vec<String>> {
        Ok(self.menu_api_bindings(menu_item_id).await?.api_permission_ids)
    }

    async fn api_menu_ids(&self, api_permission_id: &str) -> RbacResult<Vec<String>> {
        Ok(self.api_menu_bindings(api_permission_id).await?.menu_item_ids)
    }

    async fn role_permission_bindings(&self, role_code: &str) -> RbacResult<RolePermissionBindingInput> {
        self.role_permission_bindings(role_code).await
    }
}
