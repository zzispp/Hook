use super::ApiDefinition;

pub const MODEL_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "models_global_read",
        method: "GET",
        path_pattern: "/api/admin/models/global",
        name: "List global models",
        group: "Models",
    },
    ApiDefinition {
        code: "models_global_create",
        method: "POST",
        path_pattern: "/api/admin/models/global",
        name: "Create global model",
        group: "Models",
    },
    ApiDefinition {
        code: "models_global_detail",
        method: "GET",
        path_pattern: "/api/admin/models/global/{id}",
        name: "Get global model",
        group: "Models",
    },
    ApiDefinition {
        code: "models_global_update",
        method: "PATCH",
        path_pattern: "/api/admin/models/global/{id}",
        name: "Update global model",
        group: "Models",
    },
    ApiDefinition {
        code: "models_global_delete",
        method: "DELETE",
        path_pattern: "/api/admin/models/global/{id}",
        name: "Delete global model",
        group: "Models",
    },
    ApiDefinition {
        code: "models_global_batch_delete",
        method: "POST",
        path_pattern: "/api/admin/models/global/batch-delete",
        name: "Batch delete global models",
        group: "Models",
    },
    ApiDefinition {
        code: "models_global_providers",
        method: "GET",
        path_pattern: "/api/admin/models/global/{id}/providers",
        name: "Global model providers",
        group: "Models",
    },
    ApiDefinition {
        code: "models_catalog_read",
        method: "GET",
        path_pattern: "/api/admin/models/catalog",
        name: "Model catalog",
        group: "Models",
    },
    ApiDefinition {
        code: "models_public_catalog_read",
        method: "GET",
        path_pattern: "/api/models/catalog",
        name: "Public model catalog",
        group: "Models",
    },
    ApiDefinition {
        code: "models_external_read",
        method: "GET",
        path_pattern: "/api/admin/models/external",
        name: "External models",
        group: "Models",
    },
];
