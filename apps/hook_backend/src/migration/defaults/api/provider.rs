use super::ApiDefinition;

pub const PROVIDER_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "providers_read",
        method: "GET",
        path_pattern: "/api/admin/providers",
        name: "提供商列表",
    },
    ApiDefinition {
        code: "providers_create",
        method: "POST",
        path_pattern: "/api/admin/providers",
        name: "创建提供商",
    },
    ApiDefinition {
        code: "providers_detail",
        method: "GET",
        path_pattern: "/api/admin/providers/{id}",
        name: "提供商详情",
    },
    ApiDefinition {
        code: "providers_update",
        method: "PATCH",
        path_pattern: "/api/admin/providers/{id}",
        name: "更新提供商",
    },
    ApiDefinition {
        code: "providers_delete",
        method: "DELETE",
        path_pattern: "/api/admin/providers/{id}",
        name: "删除提供商",
    },
    ApiDefinition {
        code: "provider_endpoints_read",
        method: "GET",
        path_pattern: "/api/admin/providers/{provider_id}/endpoints",
        name: "提供商端点列表",
    },
    ApiDefinition {
        code: "provider_endpoints_create",
        method: "POST",
        path_pattern: "/api/admin/providers/{provider_id}/endpoints",
        name: "创建提供商端点",
    },
    ApiDefinition {
        code: "provider_keys_read",
        method: "GET",
        path_pattern: "/api/admin/providers/{provider_id}/keys",
        name: "提供商密钥列表",
    },
    ApiDefinition {
        code: "provider_keys_create",
        method: "POST",
        path_pattern: "/api/admin/providers/{provider_id}/keys",
        name: "创建提供商密钥",
    },
    ApiDefinition {
        code: "provider_keys_update",
        method: "PATCH",
        path_pattern: "/api/admin/providers/{provider_id}/keys/{key_id}",
        name: "更新提供商密钥",
    },
    ApiDefinition {
        code: "provider_keys_delete",
        method: "DELETE",
        path_pattern: "/api/admin/providers/{provider_id}/keys/{key_id}",
        name: "删除提供商密钥",
    },
    ApiDefinition {
        code: "provider_models_read",
        method: "GET",
        path_pattern: "/api/admin/providers/{provider_id}/models",
        name: "提供商模型绑定",
    },
    ApiDefinition {
        code: "provider_models_create",
        method: "POST",
        path_pattern: "/api/admin/providers/{provider_id}/models",
        name: "创建提供商模型绑定",
    },
    ApiDefinition {
        code: "provider_models_update",
        method: "PATCH",
        path_pattern: "/api/admin/providers/{provider_id}/models/{model_id}",
        name: "更新提供商模型绑定",
    },
    ApiDefinition {
        code: "provider_models_delete",
        method: "DELETE",
        path_pattern: "/api/admin/providers/{provider_id}/models/{model_id}",
        name: "删除提供商模型绑定",
    },
    ApiDefinition {
        code: "provider_cooldowns_read",
        method: "GET",
        path_pattern: "/api/admin/provider-cooldowns",
        name: "提供商冷却列表",
    },
    ApiDefinition {
        code: "provider_cooldowns_release",
        method: "POST",
        path_pattern: "/api/admin/provider-cooldowns/{provider_id}/release",
        name: "解除提供商冷却",
    },
    ApiDefinition {
        code: "request_records_read",
        method: "GET",
        path_pattern: "/api/admin/request-records",
        name: "请求记录列表",
    },
    ApiDefinition {
        code: "request_records_active_read",
        method: "POST",
        path_pattern: "/api/admin/request-records/active",
        name: "活跃请求记录",
    },
    ApiDefinition {
        code: "request_records_detail",
        method: "GET",
        path_pattern: "/api/admin/request-records/{request_id}",
        name: "请求记录详情",
    },
];
