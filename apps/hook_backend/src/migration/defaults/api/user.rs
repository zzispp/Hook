use super::ApiDefinition;

pub const USER_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "users_read",
        method: "GET",
        path_pattern: "/api/users",
        name: "List users",
        group: "Users",
    },
    ApiDefinition {
        code: "users_create",
        method: "POST",
        path_pattern: "/api/users",
        name: "Create user",
        group: "Users",
    },
    ApiDefinition {
        code: "users_update",
        method: "PUT",
        path_pattern: "/api/users/{id}",
        name: "Update user",
        group: "Users",
    },
    ApiDefinition {
        code: "users_delete",
        method: "DELETE",
        path_pattern: "/api/users/{id}",
        name: "Delete user",
        group: "Users",
    },
];
