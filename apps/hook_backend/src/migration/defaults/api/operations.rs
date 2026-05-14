use super::ApiDefinition;

pub const OPERATIONS_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "announcements_read",
        method: "GET",
        path_pattern: "/api/announcements",
        name: "公告列表",
    },
    ApiDefinition {
        code: "announcements_detail",
        method: "GET",
        path_pattern: "/api/announcements/{id}",
        name: "公告详情",
    },
    ApiDefinition {
        code: "admin_announcements_read",
        method: "GET",
        path_pattern: "/api/admin/announcements",
        name: "公告管理列表",
    },
    ApiDefinition {
        code: "admin_announcements_create",
        method: "POST",
        path_pattern: "/api/admin/announcements",
        name: "创建公告",
    },
    ApiDefinition {
        code: "admin_announcements_detail",
        method: "GET",
        path_pattern: "/api/admin/announcements/{id}",
        name: "公告管理详情",
    },
    ApiDefinition {
        code: "admin_announcements_update",
        method: "PATCH",
        path_pattern: "/api/admin/announcements/{id}",
        name: "更新公告",
    },
    ApiDefinition {
        code: "admin_announcements_delete",
        method: "DELETE",
        path_pattern: "/api/admin/announcements/{id}",
        name: "删除公告",
    },
    ApiDefinition {
        code: "tickets_read",
        method: "GET",
        path_pattern: "/api/tickets",
        name: "我的工单列表",
    },
    ApiDefinition {
        code: "tickets_create",
        method: "POST",
        path_pattern: "/api/tickets",
        name: "创建我的工单",
    },
    ApiDefinition {
        code: "tickets_detail",
        method: "GET",
        path_pattern: "/api/tickets/{id}",
        name: "我的工单详情",
    },
    ApiDefinition {
        code: "tickets_reply",
        method: "PATCH",
        path_pattern: "/api/tickets/{id}/messages",
        name: "回复我的工单",
    },
    ApiDefinition {
        code: "admin_tickets_read",
        method: "GET",
        path_pattern: "/api/admin/tickets",
        name: "工单管理列表",
    },
    ApiDefinition {
        code: "admin_tickets_detail",
        method: "GET",
        path_pattern: "/api/admin/tickets/{id}",
        name: "工单管理详情",
    },
    ApiDefinition {
        code: "admin_tickets_update",
        method: "PATCH",
        path_pattern: "/api/admin/tickets/{id}",
        name: "更新工单",
    },
    ApiDefinition {
        code: "admin_tickets_reply",
        method: "PATCH",
        path_pattern: "/api/admin/tickets/{id}/messages",
        name: "回复工单",
    },
    ApiDefinition {
        code: "notifications_read",
        method: "GET",
        path_pattern: "/api/notifications",
        name: "通知列表",
    },
    ApiDefinition {
        code: "notifications_read_all",
        method: "PATCH",
        path_pattern: "/api/notifications/read-all",
        name: "全部通知已读",
    },
    ApiDefinition {
        code: "notification_read",
        method: "PATCH",
        path_pattern: "/api/notifications/{source_type}/{source_id}/read",
        name: "通知已读",
    },
    ApiDefinition {
        code: "notification_delete",
        method: "DELETE",
        path_pattern: "/api/notifications/{source_type}/{source_id}",
        name: "删除通知",
    },
];
