# Progress

## Recovery

任务: 修复请求记录 toolbar 状态筛选下拉
形态: single-full
进度: 3/3
当前: 已完成
文件: `.codex-tasks/20260512-request-record-toolbar-filter/TODO.csv`
下一步: 读取请求记录 view/table/toolbar 与过滤工具函数。

## Log

- 2026-05-12: 创建任务文件。
- 2026-05-12: 定位到 `apps/hook_frontend/src/sections/admin/request-records-view.tsx` 中全部状态使用空字符串值，MUI Select 空值显示为空白；过滤参数清理逻辑本身正确。
- 2026-05-12: 将全部状态改为 `all` UI 哨兵值，并在 onChange 时转换回空字符串。
- 2026-05-12: `pnpm lint:frontend` 通过。
