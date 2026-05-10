use super::ApiDefinition;

pub const AUTH_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "auth_me",
        method: "GET",
        path_pattern: "/api/auth/me",
        name: "当前用户",
    },
    ApiDefinition {
        code: "navbar_read",
        method: "GET",
        path_pattern: "/api/navbar",
        name: "导航菜单",
    },
];
