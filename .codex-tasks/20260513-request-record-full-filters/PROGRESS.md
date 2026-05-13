# Progress

## Recovery

任务: Request record full filters.
形态: single-full
进度: 1/6
当前: Implement backend all-record filters.
文件: `.codex-tasks/20260513-request-record-full-filters/TODO.csv`
下一步: 补齐前端滚动耗时显示后进行当前轮验证。

## Log

- 2026-05-13: 已确认现状不是前端当前页筛选；前端把 `search/status/skip/limit` 传给后端。
- 2026-05-13: 已确认后端当前实现先取最近 `1000` 条 `request_candidates`，聚合成请求记录后筛选分页，因此不是严格全库筛选。
- 2026-05-13: 恢复时发现后端汇总表、列表筛选、前端筛选栏代码已存在；本轮继续补用户新增的首字/总耗时实时滚动显示。
