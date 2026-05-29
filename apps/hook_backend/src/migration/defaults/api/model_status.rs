use super::ApiDefinition;

pub const MODEL_STATUS_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "model_status_checks_read",
        method: "GET",
        path_pattern: "/api/model-status/checks",
        name: "读取模型状态",
    },
    ApiDefinition {
        code: "admin_model_status_checks_read",
        method: "GET",
        path_pattern: "/api/admin/model-status/checks",
        name: "模型状态检查列表",
    },
    ApiDefinition {
        code: "admin_model_status_checks_create",
        method: "POST",
        path_pattern: "/api/admin/model-status/checks",
        name: "创建模型状态检查",
    },
    ApiDefinition {
        code: "admin_model_status_checks_batch_create",
        method: "POST",
        path_pattern: "/api/admin/model-status/checks/batch-create",
        name: "批量创建模型状态检查",
    },
    ApiDefinition {
        code: "admin_model_status_checks_update",
        method: "PATCH",
        path_pattern: "/api/admin/model-status/checks/{id}",
        name: "更新模型状态检查",
    },
    ApiDefinition {
        code: "admin_model_status_checks_delete",
        method: "DELETE",
        path_pattern: "/api/admin/model-status/checks/{id}",
        name: "删除模型状态检查",
    },
    ApiDefinition {
        code: "admin_model_status_checks_batch_delete",
        method: "POST",
        path_pattern: "/api/admin/model-status/checks/batch-delete",
        name: "批量删除模型状态检查",
    },
    ApiDefinition {
        code: "admin_model_status_checks_batch_update",
        method: "POST",
        path_pattern: "/api/admin/model-status/checks/batch-update",
        name: "批量更新模型状态检查",
    },
    ApiDefinition {
        code: "admin_model_status_runs_read",
        method: "GET",
        path_pattern: "/api/admin/model-status/runs",
        name: "模型状态探测记录",
    },
];
