use super::ApiDefinition;

pub const PERFORMANCE_MONITORING_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "performance_monitoring_overview_read",
        method: "GET",
        path_pattern: "/api/admin/performance-monitoring/overview",
        name: "读取性能监控概览",
    },
    ApiDefinition {
        code: "performance_monitoring_realtime_read",
        method: "GET",
        path_pattern: "/api/admin/performance-monitoring/realtime",
        name: "读取性能监控实时指标",
    },
];
