use super::ApiDefinition;

pub const GROUP_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "groups_read",
        method: "GET",
        path_pattern: "/api/admin/groups",
        name: "计费分组列表",
    },
    ApiDefinition {
        code: "groups_create",
        method: "POST",
        path_pattern: "/api/admin/groups",
        name: "创建计费分组",
    },
    ApiDefinition {
        code: "groups_detail",
        method: "GET",
        path_pattern: "/api/admin/groups/{id}",
        name: "计费分组详情",
    },
    ApiDefinition {
        code: "groups_update",
        method: "PATCH",
        path_pattern: "/api/admin/groups/{id}",
        name: "更新计费分组",
    },
    ApiDefinition {
        code: "groups_delete",
        method: "DELETE",
        path_pattern: "/api/admin/groups/{id}",
        name: "删除计费分组",
    },
    ApiDefinition {
        code: "groups_available_read",
        method: "GET",
        path_pattern: "/api/groups/available",
        name: "可用计费分组",
    },
];
