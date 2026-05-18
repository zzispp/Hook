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
    ApiDefinition {
        code: "password_reset_request",
        method: "POST",
        path_pattern: "/api/auth/password-reset/request",
        name: "请求找回密码",
    },
    ApiDefinition {
        code: "password_reset_confirm",
        method: "POST",
        path_pattern: "/api/auth/password-reset/confirm",
        name: "重置密码",
    },
];
