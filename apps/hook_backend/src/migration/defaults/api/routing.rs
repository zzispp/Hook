use super::ApiDefinition;

pub const ROUTING_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "routing_profiles_read",
        method: "GET",
        path_pattern: "/api/admin/routing/profiles",
        name: "读取路由策略配置",
    },
    ApiDefinition {
        code: "routing_profiles_update",
        method: "PUT",
        path_pattern: "/api/admin/routing/profiles/{id}",
        name: "更新路由策略配置",
    },
    ApiDefinition {
        code: "routing_rankings_read",
        method: "GET",
        path_pattern: "/api/admin/routing/rankings",
        name: "读取路由策略排序",
    },
    ApiDefinition {
        code: "routing_decision_read",
        method: "GET",
        path_pattern: "/api/admin/routing/decisions/{request_id}",
        name: "读取路由决策详情",
    },
    ApiDefinition {
        code: "routing_preview",
        method: "POST",
        path_pattern: "/api/admin/routing/preview",
        name: "预览路由策略排序",
    },
];
