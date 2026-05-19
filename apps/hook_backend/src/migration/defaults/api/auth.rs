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
        code: "auth_config",
        method: "GET",
        path_pattern: "/api/auth/config",
        name: "认证配置",
    },
    ApiDefinition {
        code: "registration_email_code_request",
        method: "POST",
        path_pattern: "/api/auth/registration-email-code",
        name: "请求注册邮件验证码",
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
