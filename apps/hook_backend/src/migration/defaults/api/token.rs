use super::ApiDefinition;

pub const TOKEN_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "api_tokens_read",
        method: "GET",
        path_pattern: "/api/tokens",
        name: "我的令牌列表",
    },
    ApiDefinition {
        code: "api_tokens_create",
        method: "POST",
        path_pattern: "/api/tokens",
        name: "创建我的令牌",
    },
    ApiDefinition {
        code: "api_tokens_detail",
        method: "GET",
        path_pattern: "/api/tokens/{id}",
        name: "我的令牌详情",
    },
    ApiDefinition {
        code: "api_tokens_update",
        method: "PATCH",
        path_pattern: "/api/tokens/{id}",
        name: "更新我的令牌",
    },
    ApiDefinition {
        code: "api_tokens_delete",
        method: "DELETE",
        path_pattern: "/api/tokens/{id}",
        name: "删除我的令牌",
    },
    ApiDefinition {
        code: "api_tokens_secret_read",
        method: "GET",
        path_pattern: "/api/tokens/{id}/secret",
        name: "读取我的令牌密钥",
    },
    ApiDefinition {
        code: "admin_api_tokens_read",
        method: "GET",
        path_pattern: "/api/admin/tokens",
        name: "令牌管理列表",
    },
    ApiDefinition {
        code: "admin_api_tokens_create",
        method: "POST",
        path_pattern: "/api/admin/tokens",
        name: "创建管理令牌",
    },
    ApiDefinition {
        code: "admin_api_tokens_detail",
        method: "GET",
        path_pattern: "/api/admin/tokens/{id}",
        name: "管理令牌详情",
    },
    ApiDefinition {
        code: "admin_api_tokens_update",
        method: "PATCH",
        path_pattern: "/api/admin/tokens/{id}",
        name: "更新管理令牌",
    },
    ApiDefinition {
        code: "admin_api_tokens_delete",
        method: "DELETE",
        path_pattern: "/api/admin/tokens/{id}",
        name: "删除管理令牌",
    },
    ApiDefinition {
        code: "admin_api_tokens_secret_read",
        method: "GET",
        path_pattern: "/api/admin/tokens/{id}/secret",
        name: "读取管理令牌密钥",
    },
];
