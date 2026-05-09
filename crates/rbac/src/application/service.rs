use types::{
    pagination::Page,
    rbac::{
        ApiMenuBindingInput, ApiPermission, ApiPermissionInput, MenuApiBindingInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse,
        RbacListRequest, Role, RoleInput, RolePermissionBindingInput,
    },
};

use crate::application::{ApiCheckRequest, AuthorizationConfig, RbacCache, RbacRepository, RbacResult};

use self::{
    authz::{authorize_snapshot, is_authenticated_base, is_whitelisted},
    guards::{
        ensure_api_permission_exists, ensure_api_permissions_exist, ensure_menu_item_exists, ensure_menu_items_exist, ensure_menu_parent_is_valid,
        ensure_menu_section_exists, ensure_role_exists, reject_bound_api_delete, reject_bound_role_delete, reject_menu_item_delete_with_dependents,
        reject_non_empty_menu_section_delete, reject_system_role_update,
    },
    validation::{sanitize_api, sanitize_menu_item, sanitize_menu_section, sanitize_role, validate_page},
};

mod authz;
mod guards;
mod use_cases;
mod validation;

pub struct RbacService<R, C> {
    repository: R,
    cache: C,
}

impl<R, C> RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    pub const fn new(repository: R, cache: C) -> Self {
        Self { repository, cache }
    }

    pub async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        let role = self.repository.create_role(sanitize_role(input)?).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn ensure_system_role(&self, input: RoleInput) -> RbacResult<Role> {
        let input = sanitize_role(input)?;
        let code = input.code.clone();
        let role = match self.repository.find_role(&code).await? {
            Some(_) => self.repository.replace_system_role(&code, input).await?,
            None => self.repository.create_system_role(input).await?,
        };
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, code).await?;
        let role = self.repository.replace_role(code, sanitize_role(input)?).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn delete_role(&self, code: &str) -> RbacResult<()> {
        reject_system_role_update(&self.repository, code).await?;
        reject_bound_role_delete(&self.repository, code).await?;
        self.repository.delete_role(code).await?;
        self.rebuild_cache().await
    }

    pub async fn list_roles(&self) -> RbacResult<Vec<Role>> {
        self.repository.list_roles().await
    }

    pub async fn page_roles(&self, request: RbacListRequest) -> RbacResult<Page<Role>> {
        validate_page(request.page)?;
        self.repository.page_roles(request).await
    }

    pub async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let input = sanitize_api(input)?;
        ensure_menu_items_exist(&self.repository, &input.menu_item_ids).await?;
        let api = self.repository.create_api(input).await?;
        self.rebuild_cache().await?;
        Ok(api)
    }

    pub async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        ensure_api_permission_exists(&self.repository, id).await?;
        let input = sanitize_api(input)?;
        ensure_menu_items_exist(&self.repository, &input.menu_item_ids).await?;
        let api = self.repository.replace_api(id, input).await?;
        self.rebuild_cache().await?;
        Ok(api)
    }

    pub async fn delete_api(&self, id: &str) -> RbacResult<()> {
        ensure_api_permission_exists(&self.repository, id).await?;
        reject_bound_api_delete(&self.repository, id).await?;
        self.repository.delete_api(id).await?;
        self.rebuild_cache().await
    }

    pub async fn list_apis(&self) -> RbacResult<Vec<ApiPermission>> {
        self.repository.list_apis().await
    }

    pub async fn page_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>> {
        validate_page(request.page)?;
        self.repository.page_apis(request).await
    }

    pub async fn page_unbound_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>> {
        validate_page(request.page)?;
        self.repository.page_unbound_apis(request).await
    }

    pub async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection> {
        let section = self.repository.create_menu_section(sanitize_menu_section(input)?).await?;
        self.rebuild_cache().await?;
        Ok(section)
    }

    pub async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection> {
        let section = self.repository.replace_menu_section(id, sanitize_menu_section(input)?).await?;
        self.rebuild_cache().await?;
        Ok(section)
    }

    pub async fn delete_menu_section(&self, id: &str) -> RbacResult<()> {
        ensure_menu_section_exists(&self.repository, id).await?;
        reject_non_empty_menu_section_delete(&self.repository, id).await?;
        self.repository.delete_menu_section(id).await?;
        self.rebuild_cache().await
    }

    pub async fn page_menu_sections(&self, request: RbacListRequest) -> RbacResult<Page<MenuSection>> {
        validate_page(request.page)?;
        self.repository.page_menu_sections(request).await
    }

    pub async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        let input = sanitize_menu_item(input)?;
        ensure_menu_section_exists(&self.repository, &input.section_id).await?;
        ensure_menu_parent_is_valid(&self.repository, None, &input).await?;
        let item = self.repository.create_menu_item(input).await?;
        self.rebuild_cache().await?;
        Ok(item)
    }

    pub async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        let input = sanitize_menu_item(input)?;
        ensure_menu_item_exists(&self.repository, id).await?;
        ensure_menu_section_exists(&self.repository, &input.section_id).await?;
        ensure_menu_parent_is_valid(&self.repository, Some(id), &input).await?;
        let item = self.repository.replace_menu_item(id, input).await?;
        self.rebuild_cache().await?;
        Ok(item)
    }

    pub async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        ensure_menu_item_exists(&self.repository, id).await?;
        reject_menu_item_delete_with_dependents(&self.repository, id).await?;
        self.repository.delete_menu_item(id).await?;
        self.rebuild_cache().await
    }

    pub async fn page_menu_items(&self, request: RbacListRequest) -> RbacResult<Page<MenuItem>> {
        validate_page(request.page)?;
        self.repository.page_menu_items(request).await
    }

    pub async fn replace_role_permissions(&self, role_code: &str, input: RolePermissionBindingInput) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_code).await?;
        ensure_menu_items_exist(&self.repository, &input.menu_item_ids).await?;
        ensure_api_permissions_exist(&self.repository, &input.api_permission_ids).await?;
        self.repository.replace_role_permissions(role_code, input).await?;
        self.rebuild_cache().await
    }

    pub async fn replace_menu_apis(&self, menu_item_id: &str, input: MenuApiBindingInput) -> RbacResult<()> {
        ensure_menu_item_exists(&self.repository, menu_item_id).await?;
        ensure_api_permissions_exist(&self.repository, &input.api_permission_ids).await?;
        self.repository.replace_menu_apis(menu_item_id, input).await?;
        self.rebuild_cache().await
    }

    pub async fn replace_api_menus(&self, api_permission_id: &str, input: ApiMenuBindingInput) -> RbacResult<()> {
        ensure_api_permission_exists(&self.repository, api_permission_id).await?;
        ensure_menu_items_exist(&self.repository, &input.menu_item_ids).await?;
        self.repository.replace_api_menus(api_permission_id, input).await?;
        self.rebuild_cache().await
    }

    pub async fn menu_api_bindings(&self, menu_item_id: &str) -> RbacResult<MenuApiBindingInput> {
        ensure_menu_item_exists(&self.repository, menu_item_id).await?;
        Ok(MenuApiBindingInput {
            api_permission_ids: self.repository.menu_api_ids(menu_item_id).await?,
        })
    }

    pub async fn api_menu_bindings(&self, api_permission_id: &str) -> RbacResult<ApiMenuBindingInput> {
        ensure_api_permission_exists(&self.repository, api_permission_id).await?;
        Ok(ApiMenuBindingInput {
            menu_item_ids: self.repository.api_menu_ids(api_permission_id).await?,
        })
    }

    pub async fn role_permission_bindings(&self, role_code: &str) -> RbacResult<RolePermissionBindingInput> {
        ensure_role_exists(&self.repository, role_code).await?;
        Ok(RolePermissionBindingInput {
            menu_item_ids: self.repository.role_menu_item_ids(role_code).await?,
            api_permission_ids: self.repository.role_api_ids(role_code).await?,
        })
    }

    pub async fn navbar(&self, role_code: &str) -> RbacResult<NavResponse> {
        self.cache.read_nav(role_code).await
    }

    pub async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()> {
        if self.is_whitelisted(config, &request.method, &request.path)? {
            return Ok(());
        }

        if request.system {
            return Ok(());
        }

        if is_authenticated_base(config, &request.method, &request.path)? {
            return Ok(());
        }

        let snapshot = self.cache.read_snapshot().await?;
        authorize_snapshot(&snapshot.api_permissions, &request)
    }

    pub fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
        is_whitelisted(config, method, path)
    }

    pub async fn rebuild_cache(&self) -> RbacResult<()> {
        let snapshot = self.repository.permission_snapshot().await?;
        self.cache.write_snapshot(&snapshot).await
    }
}

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
