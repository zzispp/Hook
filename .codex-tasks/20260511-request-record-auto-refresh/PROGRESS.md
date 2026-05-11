# Progress

## Recovery

- 任务: 对照 aether 给 Hook 补齐请求记录自动刷新
- 形态: single-full
- 进度: 3/4
- 当前: 校验并汇总结论
- 文件: `.codex-tasks/20260511-request-record-auto-refresh/TODO.csv`
- 下一步: 搜索 aether 中 `active-requests` 与请求记录自动刷新调用链

## Log

- 2026-05-11: 创建任务记录，开始只读调查。
- 2026-05-11: 确认 aether 的 `frontend/src/views/shared/Usage.vue` 使用 1 秒 `active-requests` 轻量轮询和 3 秒全量 records 刷新；Hook 的 `request-records-view.tsx` 目前只有 3 秒列表刷新。
- 2026-05-11: Hook 已新增 `/api/admin/request-records/active` 与前端 1 秒活跃请求轮询；未加入人为 ID 上限或静默 fallback。
