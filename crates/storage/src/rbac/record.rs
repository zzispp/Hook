use types::rbac::{ApiPermission, MenuItem, MenuSection, Role};

#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{api_permissions, menu_items, menu_sections, role_api_permissions, role_menu_permissions, roles};

pub type RoleRecord = roles::Model;
pub type ApiPermissionRecord = api_permissions::Model;
pub type MenuSectionRecord = menu_sections::Model;
pub type MenuItemRecord = menu_items::Model;
pub type RoleApiPermissionRecord = role_api_permissions::Model;
pub type RoleMenuPermissionRecord = role_menu_permissions::Model;

impl From<RoleRecord> for Role {
    fn from(value: RoleRecord) -> Self {
        Self {
            code: value.code,
            name: value.name,
            description: value.description,
            enabled: value.enabled,
            system: value.system,
            sort_order: value.sort_order,
        }
    }
}

impl From<ApiPermissionRecord> for ApiPermission {
    fn from(value: ApiPermissionRecord) -> Self {
        Self {
            id: value.id,
            code: value.code,
            method: value.method,
            path_pattern: value.path_pattern,
            name: value.name,
            group: value.group,
            enabled: value.enabled,
            system: value.system,
        }
    }
}

impl From<MenuSectionRecord> for MenuSection {
    fn from(value: MenuSectionRecord) -> Self {
        Self {
            id: value.id,
            code: value.code,
            subheader: value.subheader,
            sort_order: value.sort_order,
            enabled: value.enabled,
        }
    }
}

impl From<MenuItemRecord> for MenuItem {
    fn from(value: MenuItemRecord) -> Self {
        Self {
            id: value.id,
            section_id: value.section_id,
            parent_id: value.parent_id,
            code: value.code,
            title: value.title,
            path: value.route_path,
            icon: value.icon,
            caption: value.caption,
            deep_match: value.deep_match,
            sort_order: value.sort_order,
            enabled: value.enabled,
        }
    }
}
