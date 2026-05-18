use super::ApiDefinition;

pub const DASHBOARD_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "dashboard_overview_read",
        method: "GET",
        path_pattern: "/api/dashboard/overview",
        name: "读取仪表盘概览",
    },
    ApiDefinition {
        code: "dashboard_activity_read",
        method: "GET",
        path_pattern: "/api/dashboard/activity",
        name: "读取仪表盘活跃网格",
    },
    ApiDefinition {
        code: "dashboard_filter_options_read",
        method: "GET",
        path_pattern: "/api/dashboard/filter-options",
        name: "读取仪表盘筛选项",
    },
];
