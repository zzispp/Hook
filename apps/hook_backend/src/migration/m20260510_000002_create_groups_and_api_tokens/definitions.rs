pub(super) const DEFAULT_GROUP_ID: &str = "00000000-0000-7000-8000-000000000401";
pub(super) const DEFAULT_GROUP_CODE: &str = constants::billing::DEFAULT_SYSTEM_GROUP_CODE;
pub(super) const API_TOKEN_MENU_ID: &str = "00000000-0000-7000-8000-000000000210";
pub(super) const ADMIN_GROUP_MENU_ID: &str = "00000000-0000-7000-8000-000000000211";
pub(super) const RESOURCES_SECTION_ID: &str = "00000000-0000-7000-8000-000000000102";
pub(super) const SYSTEM_SECTION_ID: &str = "00000000-0000-7000-8000-000000000103";
pub(super) const ADMIN_ROLE: &str = "admin";
pub(super) const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;
pub(super) const API_BASE_ID: usize = 401;

pub(super) struct ApiDefinition {
    pub code: &'static str,
    pub method: &'static str,
    pub path_pattern: &'static str,
    pub name: &'static str,
    pub group: &'static str,
}

pub(super) struct MenuDefinition {
    pub id: &'static str,
    pub section_id: &'static str,
    pub code: &'static str,
    pub title: &'static str,
    pub path: &'static str,
    pub icon: &'static str,
    pub sort_order: i64,
}

pub(super) struct MenuApiBinding {
    pub menu_id: &'static str,
    pub api_codes: &'static [&'static str],
}

pub(super) const MENU_DEFINITIONS: &[MenuDefinition] = &[
    MenuDefinition {
        id: API_TOKEN_MENU_ID,
        section_id: RESOURCES_SECTION_ID,
        code: "api_tokens",
        title: "API Tokens",
        path: "/dashboard/tokens",
        icon: "icon.key",
        sort_order: 10,
    },
    MenuDefinition {
        id: ADMIN_GROUP_MENU_ID,
        section_id: SYSTEM_SECTION_ID,
        code: "admin_groups",
        title: "Billing Groups",
        path: "/dashboard/admin/groups",
        icon: "icon.group",
        sort_order: 60,
    },
];

pub(super) const API_DEFINITIONS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "groups_read",
        method: "GET",
        path_pattern: "/api/admin/groups",
        name: "List billing groups",
        group: "Billing Groups",
    },
    ApiDefinition {
        code: "groups_create",
        method: "POST",
        path_pattern: "/api/admin/groups",
        name: "Create billing group",
        group: "Billing Groups",
    },
    ApiDefinition {
        code: "groups_detail",
        method: "GET",
        path_pattern: "/api/admin/groups/{id}",
        name: "Get billing group",
        group: "Billing Groups",
    },
    ApiDefinition {
        code: "groups_update",
        method: "PATCH",
        path_pattern: "/api/admin/groups/{id}",
        name: "Update billing group",
        group: "Billing Groups",
    },
    ApiDefinition {
        code: "groups_delete",
        method: "DELETE",
        path_pattern: "/api/admin/groups/{id}",
        name: "Delete billing group",
        group: "Billing Groups",
    },
    ApiDefinition {
        code: "groups_available_read",
        method: "GET",
        path_pattern: "/api/groups/available",
        name: "Available billing groups",
        group: "Billing Groups",
    },
    ApiDefinition {
        code: "api_tokens_read",
        method: "GET",
        path_pattern: "/api/tokens",
        name: "List API tokens",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "api_tokens_create",
        method: "POST",
        path_pattern: "/api/tokens",
        name: "Create API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "api_tokens_detail",
        method: "GET",
        path_pattern: "/api/tokens/{id}",
        name: "Get API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "api_tokens_update",
        method: "PATCH",
        path_pattern: "/api/tokens/{id}",
        name: "Update API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "api_tokens_delete",
        method: "DELETE",
        path_pattern: "/api/tokens/{id}",
        name: "Delete API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "api_tokens_secret_read",
        method: "GET",
        path_pattern: "/api/tokens/{id}/secret",
        name: "Read API token secret",
        group: "API Tokens",
    },
];

pub(super) const MENU_API_BINDINGS: &[MenuApiBinding] = &[
    MenuApiBinding {
        menu_id: API_TOKEN_MENU_ID,
        api_codes: &[
            "groups_available_read",
            "models_public_catalog_read",
            "api_tokens_read",
            "api_tokens_create",
            "api_tokens_detail",
            "api_tokens_update",
            "api_tokens_delete",
            "api_tokens_secret_read",
        ],
    },
    MenuApiBinding {
        menu_id: ADMIN_GROUP_MENU_ID,
        api_codes: &["groups_read", "groups_create", "groups_detail", "groups_update", "groups_delete"],
    },
];
