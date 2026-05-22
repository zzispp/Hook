use super::ApiDefinition;

pub const SCHEDULER_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "scheduled_tasks_read",
        method: "GET",
        path_pattern: "/api/admin/scheduled-tasks",
        name: "定时任务列表",
    },
    ApiDefinition {
        code: "scheduled_tasks_update",
        method: "PATCH",
        path_pattern: "/api/admin/scheduled-tasks/{code}",
        name: "更新定时任务",
    },
    ApiDefinition {
        code: "scheduled_task_runs_read",
        method: "GET",
        path_pattern: "/api/admin/scheduled-task-runs",
        name: "定时任务执行记录",
    },
];
