use super::ApiDefinition;

pub const MODEL_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "models_global_read",
        method: "GET",
        path_pattern: "/api/admin/models/global",
        name: "模型列表",
    },
    ApiDefinition {
        code: "models_global_create",
        method: "POST",
        path_pattern: "/api/admin/models/global",
        name: "创建模型",
    },
    ApiDefinition {
        code: "models_global_detail",
        method: "GET",
        path_pattern: "/api/admin/models/global/{id}",
        name: "模型详情",
    },
    ApiDefinition {
        code: "models_global_update",
        method: "PATCH",
        path_pattern: "/api/admin/models/global/{id}",
        name: "更新模型",
    },
    ApiDefinition {
        code: "models_global_delete",
        method: "DELETE",
        path_pattern: "/api/admin/models/global/{id}",
        name: "删除模型",
    },
    ApiDefinition {
        code: "models_global_batch_delete",
        method: "POST",
        path_pattern: "/api/admin/models/global/batch-delete",
        name: "批量删除模型",
    },
    ApiDefinition {
        code: "models_global_providers",
        method: "GET",
        path_pattern: "/api/admin/models/global/{id}/providers",
        name: "模型供应商",
    },
    ApiDefinition {
        code: "models_catalog_read",
        method: "GET",
        path_pattern: "/api/admin/models/catalog",
        name: "模型目录",
    },
    ApiDefinition {
        code: "models_public_catalog_read",
        method: "GET",
        path_pattern: "/api/models/catalog",
        name: "用户模型目录",
    },
    ApiDefinition {
        code: "models_external_read",
        method: "GET",
        path_pattern: "/api/admin/models/external",
        name: "外部模型",
    },
];
