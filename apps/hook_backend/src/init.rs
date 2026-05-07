use configuration::Settings;
use rbac::application::{RbacCache, RbacError, RbacRepository, RbacService};
use types::{
    pagination::PageRequest,
    rbac::{ApiPermissionInput, MenuItemInput, MenuSectionInput, RoleInput, RoleMenuBindingInput},
};

use crate::BackendResult;

pub async fn ensure_default_rbac<R, C>(rbac: &RbacService<R, C>, settings: &Settings) -> BackendResult<()>
where
    R: RbacRepository,
    C: RbacCache,
{
    let admin_role = settings.admin.role.trim();
    rbac.ensure_system_role(RoleInput {
        code: admin_role.into(),
        name: "Administrator".into(),
        description: "Built-in administrator role".into(),
        enabled: true,
        sort_order: 0,
    })
    .await?;

    let user_role = constants::auth::DEFAULT_USER_ROLE;
    rbac.ensure_system_role(RoleInput {
        code: user_role.into(),
        name: "User".into(),
        description: "Default signed-up user role".into(),
        enabled: true,
        sort_order: 10,
    })
    .await?;

    let api_ids = ensure_default_apis(rbac).await?;
    let menu_item_ids = ensure_default_menus(rbac).await?;
    rbac.replace_role_apis(admin_role, api_ids).await?;
    rbac.replace_role_menus(admin_role, RoleMenuBindingInput { menu_item_ids }).await?;
    Ok(())
}

async fn ensure_default_apis<R, C>(rbac: &RbacService<R, C>) -> BackendResult<Vec<String>>
where
    R: RbacRepository,
    C: RbacCache,
{
    let existing = rbac.list_apis().await?;
    let mut api_ids = existing.iter().map(|api| api.id.clone()).collect::<Vec<_>>();

    for input in default_api_permissions() {
        if existing.iter().any(|api| api.code == input.code) {
            continue;
        }

        match rbac.create_api(input).await {
            Ok(api) => api_ids.push(api.id),
            Err(RbacError::Infrastructure(message)) if message.contains("duplicate key") => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(api_ids)
}

async fn ensure_default_menus<R, C>(rbac: &RbacService<R, C>) -> BackendResult<Vec<String>>
where
    R: RbacRepository,
    C: RbacCache,
{
    let sections = rbac.page_menu_sections(PageRequest { page: 1, page_size: 100 }).await?.items;
    let overview_section_id = match sections.iter().find(|section| section.code == "overview") {
        Some(section) => section.id.clone(),
        None => match rbac
            .create_menu_section(MenuSectionInput {
                code: "overview".into(),
                subheader: "Overview".into(),
                sort_order: -10,
                enabled: true,
            })
            .await
        {
            Ok(section) => section.id,
            Err(RbacError::Infrastructure(message)) if message.contains("duplicate key") => rbac
                .page_menu_sections(PageRequest { page: 1, page_size: 100 })
                .await?
                .items
                .into_iter()
                .find(|section| section.code == "overview")
                .map(|section| section.id)
                .ok_or("default overview menu section was not found after duplicate insert")?,
            Err(error) => return Err(error.into()),
        },
    };

    let section_id = match sections.into_iter().find(|section| section.code == "system_management") {
        Some(section) => section.id,
        None => match rbac
            .create_menu_section(MenuSectionInput {
                code: "system_management".into(),
                subheader: "System Management".into(),
                sort_order: 0,
                enabled: true,
            })
            .await
        {
            Ok(section) => section.id,
            Err(RbacError::Infrastructure(message)) if message.contains("duplicate key") => rbac
                .page_menu_sections(PageRequest { page: 1, page_size: 100 })
                .await?
                .items
                .into_iter()
                .find(|section| section.code == "system_management")
                .map(|section| section.id)
                .ok_or("default menu section was not found after duplicate insert")?,
            Err(error) => return Err(error.into()),
        },
    };

    let existing = rbac.page_menu_items(PageRequest { page: 1, page_size: 100 }).await?.items;
    let mut menu_item_ids = existing.iter().map(|item| item.id.clone()).collect::<Vec<_>>();

    for input in default_menu_items(&overview_section_id, &section_id) {
        if existing.iter().any(|item| item.code == input.code) {
            continue;
        }

        match rbac.create_menu_item(input).await {
            Ok(item) => menu_item_ids.push(item.id),
            Err(RbacError::Infrastructure(message)) if message.contains("duplicate key") => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(menu_item_ids)
}

fn default_api_permissions() -> Vec<ApiPermissionInput> {
    vec![
        api_permission("auth_me", "GET", "/api/auth/me", "Current user", "Auth"),
        api_permission("navbar_read", "GET", "/api/navbar", "Navbar", "System"),
        api_permission("users_read", "GET", "/api/users", "List users", "Users"),
        api_permission("users_create", "POST", "/api/users", "Create user", "Users"),
        api_permission("users_update", "PUT", "/api/users/{id}", "Update user", "Users"),
        api_permission("users_delete", "DELETE", "/api/users/{id}", "Delete user", "Users"),
        api_permission("roles_read", "GET", "/api/rbac/roles", "List roles", "RBAC"),
        api_permission("roles_create", "POST", "/api/rbac/roles", "Create role", "RBAC"),
        api_permission("roles_update", "PUT", "/api/rbac/roles/{code}", "Update role", "RBAC"),
        api_permission("roles_delete", "DELETE", "/api/rbac/roles/{code}", "Delete role", "RBAC"),
        api_permission("role_apis_read", "GET", "/api/rbac/roles/{code}/apis", "Read role API bindings", "RBAC"),
        api_permission("role_apis_update", "PUT", "/api/rbac/roles/{code}/apis", "Update role API bindings", "RBAC"),
        api_permission("role_menus_read", "GET", "/api/rbac/roles/{code}/menus", "Read role menu bindings", "RBAC"),
        api_permission("role_menus_update", "PUT", "/api/rbac/roles/{code}/menus", "Update role menu bindings", "RBAC"),
        api_permission("apis_read", "GET", "/api/rbac/apis", "List API permissions", "RBAC"),
        api_permission("apis_create", "POST", "/api/rbac/apis", "Create API permission", "RBAC"),
        api_permission("apis_update", "PUT", "/api/rbac/apis/{id}", "Update API permission", "RBAC"),
        api_permission("apis_delete", "DELETE", "/api/rbac/apis/{id}", "Delete API permission", "RBAC"),
        api_permission("menu_sections_read", "GET", "/api/rbac/menu-sections", "List menu sections", "Menus"),
        api_permission("menu_sections_create", "POST", "/api/rbac/menu-sections", "Create menu section", "Menus"),
        api_permission("menu_sections_update", "PUT", "/api/rbac/menu-sections/{id}", "Update menu section", "Menus"),
        api_permission("menu_sections_delete", "DELETE", "/api/rbac/menu-sections/{id}", "Delete menu section", "Menus"),
        api_permission("menu_items_read", "GET", "/api/rbac/menu-items", "List menu items", "Menus"),
        api_permission("menu_items_create", "POST", "/api/rbac/menu-items", "Create menu item", "Menus"),
        api_permission("menu_items_update", "PUT", "/api/rbac/menu-items/{id}", "Update menu item", "Menus"),
        api_permission("menu_items_delete", "DELETE", "/api/rbac/menu-items/{id}", "Delete menu item", "Menus"),
    ]
}

fn api_permission(code: &str, method: &str, path_pattern: &str, name: &str, group: &str) -> ApiPermissionInput {
    ApiPermissionInput {
        code: code.into(),
        method: method.into(),
        path_pattern: path_pattern.into(),
        name: name.into(),
        group: group.into(),
        enabled: true,
    }
}

fn default_menu_items(overview_section_id: &str, section_id: &str) -> Vec<MenuItemInput> {
    vec![
        menu_item_exact(overview_section_id, "dashboard_home", "Dashboard", "/dashboard", "icon.dashboard", 0),
        menu_item(section_id, "admin_users", "User Management", "/dashboard/admin/users", "icon.user", 0),
        menu_item(section_id, "admin_roles", "Role Management", "/dashboard/admin/roles", "icon.lock", 10),
        menu_item(section_id, "admin_apis", "API Management", "/dashboard/admin/apis", "icon.menu", 20),
        menu_item(section_id, "admin_menus", "Menu Management", "/dashboard/admin/menus", "icon.menu", 30),
    ]
}

fn menu_item(section_id: &str, code: &str, title: &str, path: &str, icon: &str, sort_order: i64) -> MenuItemInput {
    menu_item_with_match(section_id, code, title, path, icon, sort_order, true)
}

fn menu_item_exact(section_id: &str, code: &str, title: &str, path: &str, icon: &str, sort_order: i64) -> MenuItemInput {
    menu_item_with_match(section_id, code, title, path, icon, sort_order, false)
}

fn menu_item_with_match(section_id: &str, code: &str, title: &str, path: &str, icon: &str, sort_order: i64, deep_match: bool) -> MenuItemInput {
    MenuItemInput {
        section_id: section_id.into(),
        parent_id: None,
        code: code.into(),
        title: title.into(),
        path: path.into(),
        icon: Some(icon.into()),
        caption: None,
        deep_match,
        sort_order,
        enabled: true,
    }
}
