use super::ApiDefinition;

pub const I18N_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "i18n_resources_read",
        method: "GET",
        path_pattern: "/api/i18n/resources",
        name: "读取翻译资源",
    },
    ApiDefinition {
        code: "admin_i18n_languages_read",
        method: "GET",
        path_pattern: "/api/admin/i18n/languages",
        name: "语言列表",
    },
    ApiDefinition {
        code: "admin_i18n_languages_create",
        method: "POST",
        path_pattern: "/api/admin/i18n/languages",
        name: "创建语言",
    },
    ApiDefinition {
        code: "admin_i18n_languages_update",
        method: "PATCH",
        path_pattern: "/api/admin/i18n/languages/{code}",
        name: "更新语言",
    },
    ApiDefinition {
        code: "admin_i18n_languages_delete",
        method: "DELETE",
        path_pattern: "/api/admin/i18n/languages/{code}",
        name: "删除语言",
    },
    ApiDefinition {
        code: "admin_i18n_translations_read",
        method: "GET",
        path_pattern: "/api/admin/i18n/translations",
        name: "翻译列表",
    },
    ApiDefinition {
        code: "admin_i18n_translations_create",
        method: "POST",
        path_pattern: "/api/admin/i18n/translations",
        name: "创建翻译",
    },
    ApiDefinition {
        code: "admin_i18n_translations_update",
        method: "PATCH",
        path_pattern: "/api/admin/i18n/translations/{id}",
        name: "更新翻译",
    },
    ApiDefinition {
        code: "admin_i18n_translations_delete",
        method: "DELETE",
        path_pattern: "/api/admin/i18n/translations/{id}",
        name: "删除翻译",
    },
    ApiDefinition {
        code: "admin_i18n_bundle_update",
        method: "PUT",
        path_pattern: "/api/admin/i18n/translations/{namespace}/{group_key}/{item_key}",
        name: "更新翻译键",
    },
];
