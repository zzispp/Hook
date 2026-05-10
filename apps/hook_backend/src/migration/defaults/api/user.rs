use super::ApiDefinition;

pub const USER_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "users_read",
        method: "GET",
        path_pattern: "/api/users",
        name: "用户列表",
    },
    ApiDefinition {
        code: "users_create",
        method: "POST",
        path_pattern: "/api/users",
        name: "创建用户",
    },
    ApiDefinition {
        code: "users_update",
        method: "PUT",
        path_pattern: "/api/users/{id}",
        name: "更新用户",
    },
    ApiDefinition {
        code: "users_delete",
        method: "DELETE",
        path_pattern: "/api/users/{id}",
        name: "删除用户",
    },
];
