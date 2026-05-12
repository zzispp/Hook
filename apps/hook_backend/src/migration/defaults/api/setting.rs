use super::ApiDefinition;

pub const SETTING_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "system_settings_read",
        method: "GET",
        path_pattern: "/api/admin/settings/system",
        name: "读取系统设置",
    },
    ApiDefinition {
        code: "system_settings_update",
        method: "PATCH",
        path_pattern: "/api/admin/settings/system",
        name: "更新系统设置",
    },
    ApiDefinition {
        code: "system_exchange_rate_read",
        method: "GET",
        path_pattern: "/api/admin/settings/exchange-rate",
        name: "读取系统汇率缓存",
    },
];
