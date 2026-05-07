use types::rbac::{ApiPermission, MenuItem, MenuSection, Role};

#[derive(Clone, Debug, toasty::Model)]
pub struct RoleRecord {
    #[key]
    pub code: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, toasty::Model)]
pub struct ApiPermissionRecord {
    #[key]
    #[column(type = varchar(36))]
    pub id: String,
    #[unique]
    pub code: String,
    pub method: String,
    pub path_pattern: String,
    pub name: String,
    pub group: String,
    pub enabled: bool,
    pub system: bool,
}

#[derive(Clone, Debug, toasty::Model)]
pub struct MenuSectionRecord {
    #[key]
    #[column(type = varchar(36))]
    pub id: String,
    #[unique]
    pub code: String,
    pub subheader: String,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, toasty::Model)]
pub struct MenuItemRecord {
    #[key]
    #[column(type = varchar(36))]
    pub id: String,
    #[index]
    #[column(type = varchar(36))]
    pub section_id: String,
    #[column(type = varchar(36))]
    pub parent_id: Option<String>,
    #[unique]
    pub code: String,
    pub title: String,
    pub route_path: String,
    pub icon: Option<String>,
    pub caption: Option<String>,
    pub deep_match: bool,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, toasty::Model)]
pub struct RoleApiPermissionRecord {
    #[key]
    pub role_code: String,
    #[key]
    #[column(type = varchar(36))]
    pub api_permission_id: String,
}

#[derive(Clone, Debug, toasty::Model)]
pub struct RoleMenuPermissionRecord {
    #[key]
    pub role_code: String,
    #[key]
    #[column(type = varchar(36))]
    pub menu_item_id: String,
}

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
