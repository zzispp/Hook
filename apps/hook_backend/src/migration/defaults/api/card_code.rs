use super::ApiDefinition;

pub const CARD_CODE_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "card_codes_redeem",
        method: "POST",
        path_pattern: "/api/card-codes/redeem",
        name: "卡密兑换",
    },
    ApiDefinition {
        code: "admin_card_codes_read",
        method: "GET",
        path_pattern: "/api/admin/card-codes",
        name: "卡密列表",
    },
    ApiDefinition {
        code: "admin_card_codes_generate",
        method: "POST",
        path_pattern: "/api/admin/card-codes/generate",
        name: "生成卡密",
    },
    ApiDefinition {
        code: "admin_card_codes_batch_status",
        method: "POST",
        path_pattern: "/api/admin/card-codes/batch-status",
        name: "批量更新卡密状态",
    },
    ApiDefinition {
        code: "admin_card_code_types_read",
        method: "GET",
        path_pattern: "/api/admin/card-code-types",
        name: "卡密类型列表",
    },
    ApiDefinition {
        code: "admin_card_code_types_create",
        method: "POST",
        path_pattern: "/api/admin/card-code-types",
        name: "创建卡密类型",
    },
    ApiDefinition {
        code: "admin_card_code_types_update",
        method: "PATCH",
        path_pattern: "/api/admin/card-code-types/{id}",
        name: "更新卡密类型",
    },
];
