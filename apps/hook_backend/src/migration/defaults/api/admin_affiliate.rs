use super::ApiDefinition;

pub const ADMIN_AFFILIATE_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "admin_affiliates_overview_read",
        method: "GET",
        path_pattern: "/api/admin/affiliates/overview",
        name: "返佣概览",
    },
    ApiDefinition {
        code: "admin_affiliates_relations_read",
        method: "GET",
        path_pattern: "/api/admin/affiliates/relations",
        name: "邀请关系列表",
    },
    ApiDefinition {
        code: "admin_affiliates_relations_update",
        method: "PATCH",
        path_pattern: "/api/admin/affiliates/relations/{user_id}",
        name: "更新邀请关系",
    },
    ApiDefinition {
        code: "admin_affiliates_relation_changes_read",
        method: "GET",
        path_pattern: "/api/admin/affiliates/relation-changes",
        name: "邀请关系变更记录",
    },
    ApiDefinition {
        code: "admin_affiliates_commissions_read",
        method: "GET",
        path_pattern: "/api/admin/affiliates/commissions",
        name: "返佣记录列表",
    },
    ApiDefinition {
        code: "admin_affiliates_reports_read",
        method: "GET",
        path_pattern: "/api/admin/affiliates/reports",
        name: "返佣报表",
    },
    ApiDefinition {
        code: "admin_affiliates_reports_export",
        method: "GET",
        path_pattern: "/api/admin/affiliates/reports/export",
        name: "导出返佣报表",
    },
];
