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
    ApiDefinition {
        code: "user_groups_read",
        method: "GET",
        path_pattern: "/api/admin/user-groups",
        name: "用户分组列表",
    },
    ApiDefinition {
        code: "user_groups_create",
        method: "POST",
        path_pattern: "/api/admin/user-groups",
        name: "创建用户分组",
    },
    ApiDefinition {
        code: "user_groups_detail",
        method: "GET",
        path_pattern: "/api/admin/user-groups/{code}",
        name: "用户分组详情",
    },
    ApiDefinition {
        code: "user_groups_update",
        method: "PATCH",
        path_pattern: "/api/admin/user-groups/{code}",
        name: "更新用户分组",
    },
    ApiDefinition {
        code: "user_groups_delete",
        method: "DELETE",
        path_pattern: "/api/admin/user-groups/{code}",
        name: "删除用户分组",
    },
    ApiDefinition {
        code: "user_groups_members_read",
        method: "GET",
        path_pattern: "/api/admin/user-groups/{code}/users",
        name: "用户分组成员",
    },
];
