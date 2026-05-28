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
    ApiDefinition {
        code: "admin_user_stats_leaderboard_read",
        method: "GET",
        path_pattern: "/api/admin/stats/leaderboard/users",
        name: "读取管理员用户统计排行榜",
    },
    ApiDefinition {
        code: "admin_user_usage_stats_read",
        method: "GET",
        path_pattern: "/api/admin/usage/stats",
        name: "读取管理员用户使用摘要",
    },
    ApiDefinition {
        code: "admin_user_stats_time_series_read",
        method: "GET",
        path_pattern: "/api/admin/stats/time-series",
        name: "读取管理员用户统计趋势",
    },
];
