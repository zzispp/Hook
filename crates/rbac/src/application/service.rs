use types::{
    pagination::{Page, PageRequest},
    rbac::{ApiPermission, ApiPermissionInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse, Role, RoleInput, RoleMenuBindingInput},
};

use crate::application::{ApiCheckRequest, AuthorizationConfig, RbacCache, RbacError, RbacRepository, RbacResult};

use self::{
    authz::{authorize_snapshot, is_whitelisted},
    validation::{sanitize_api, sanitize_menu_item, sanitize_menu_section, sanitize_role, validate_page},
};

mod authz;
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

    pub async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, code).await?;
        let role = self.repository.replace_role(code, sanitize_role(input)?).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn delete_role(&self, code: &str) -> RbacResult<()> {
        reject_system_role_update(&self.repository, code).await?;
        self.repository.delete_role(code).await?;
        self.rebuild_cache().await
    }

    pub async fn list_roles(&self) -> RbacResult<Vec<Role>> {
        self.repository.list_roles().await
    }

    pub async fn page_roles(&self, page: PageRequest) -> RbacResult<Page<Role>> {
        validate_page(page)?;
        self.repository.page_roles(page).await
    }

    pub async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let api = self.repository.create_api(sanitize_api(input)?).await?;
        self.rebuild_cache().await?;
        Ok(api)
    }

    pub async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let api = self.repository.replace_api(id, sanitize_api(input)?).await?;
        self.rebuild_cache().await?;
        Ok(api)
    }

    pub async fn delete_api(&self, id: &str) -> RbacResult<()> {
        self.repository.delete_api(id).await?;
        self.rebuild_cache().await
    }

    pub async fn list_apis(&self) -> RbacResult<Vec<ApiPermission>> {
        self.repository.list_apis().await
    }

    pub async fn page_apis(&self, page: PageRequest) -> RbacResult<Page<ApiPermission>> {
        validate_page(page)?;
        self.repository.page_apis(page).await
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
        self.repository.delete_menu_section(id).await?;
        self.rebuild_cache().await
    }

    pub async fn page_menu_sections(&self, page: PageRequest) -> RbacResult<Page<MenuSection>> {
        validate_page(page)?;
        self.repository.page_menu_sections(page).await
    }

    pub async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        let item = self.repository.create_menu_item(sanitize_menu_item(input)?).await?;
        self.rebuild_cache().await?;
        Ok(item)
    }

    pub async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        let item = self.repository.replace_menu_item(id, sanitize_menu_item(input)?).await?;
        self.rebuild_cache().await?;
        Ok(item)
    }

    pub async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        self.repository.delete_menu_item(id).await?;
        self.rebuild_cache().await
    }

    pub async fn page_menu_items(&self, page: PageRequest) -> RbacResult<Page<MenuItem>> {
        validate_page(page)?;
        self.repository.page_menu_items(page).await
    }

    pub async fn replace_role_apis(&self, role_code: &str, api_permission_ids: Vec<String>) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_code).await?;
        self.repository.replace_role_apis(role_code, api_permission_ids).await?;
        self.rebuild_cache().await
    }

    pub async fn replace_role_menus(&self, role_code: &str, input: RoleMenuBindingInput) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_code).await?;
        self.repository.replace_role_menus(role_code, input).await?;
        self.rebuild_cache().await
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

async fn reject_system_role_update<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    let role = repository.find_role(code).await?.ok_or(RbacError::NotFound)?;
    if role.system {
        return Err(RbacError::Conflict("system role cannot be changed".into()));
    }
    Ok(())
}

async fn ensure_role_exists<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    repository.find_role(code).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
