use super::ApiDefinition;

pub const CACHE_MONITORING_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "cache_monitoring_affinities_read",
        method: "GET",
        path_pattern: "/api/admin/monitoring/cache/affinities",
        name: "读取缓存亲和列表",
    },
    ApiDefinition {
        code: "cache_monitoring_affinity_delete",
        method: "DELETE",
        path_pattern: "/api/admin/monitoring/cache/affinities/{affinity_key}/{endpoint_id}/{model_id}/{api_format}",
        name: "删除缓存亲和记录",
    },
    ApiDefinition {
        code: "cache_monitoring_affinities_clear",
        method: "DELETE",
        path_pattern: "/api/admin/monitoring/cache",
        name: "清空缓存亲和记录",
    },
];
