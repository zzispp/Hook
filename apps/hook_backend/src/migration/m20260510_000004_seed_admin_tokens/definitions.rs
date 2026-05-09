pub(super) const ADMIN_TOKEN_MENU_ID: &str = "00000000-0000-7000-8000-000000000212";
pub(super) const SYSTEM_SECTION_ID: &str = "00000000-0000-7000-8000-000000000103";
pub(super) const ADMIN_ROLE: &str = "admin";
pub(super) const API_BASE_ID: usize = 421;

pub(super) struct ApiDefinition {
    pub code: &'static str,
    pub method: &'static str,
    pub path_pattern: &'static str,
    pub name: &'static str,
    pub group: &'static str,
}

pub(super) const API_DEFINITIONS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "admin_api_tokens_read",
        method: "GET",
        path_pattern: "/api/admin/tokens",
        name: "List admin API tokens",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "admin_api_tokens_create",
        method: "POST",
        path_pattern: "/api/admin/tokens",
        name: "Create admin API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "admin_api_tokens_detail",
        method: "GET",
        path_pattern: "/api/admin/tokens/{id}",
        name: "Get admin API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "admin_api_tokens_update",
        method: "PATCH",
        path_pattern: "/api/admin/tokens/{id}",
        name: "Update admin API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "admin_api_tokens_delete",
        method: "DELETE",
        path_pattern: "/api/admin/tokens/{id}",
        name: "Delete admin API token",
        group: "API Tokens",
    },
    ApiDefinition {
        code: "admin_api_tokens_secret_read",
        method: "GET",
        path_pattern: "/api/admin/tokens/{id}/secret",
        name: "Read admin API token secret",
        group: "API Tokens",
    },
];

pub(super) const ADMIN_TOKEN_API_CODES: &[&str] = &[
    "groups_available_read",
    "models_public_catalog_read",
    "users_read",
    "admin_api_tokens_read",
    "admin_api_tokens_create",
    "admin_api_tokens_detail",
    "admin_api_tokens_update",
    "admin_api_tokens_delete",
    "admin_api_tokens_secret_read",
];
