# Epic Specification

## Goal

- 重构 Hook 的请求记录体系，使其具备权威主记录、明确终态、客户端与 Provider 双视角 payload、可解释错误信息，以及可恢复的流式/WS 落库链路。

## Non-Goals

- 不引入新的外部存储系统。
- 不改动现有 provider 调度策略与计费公式本身。
- 不做兼容旧错误语义的静默 fallback。

## Constraints

- 必须遵守当前 Rust / pnpm monorepo 结构，不新增外部依赖。
- 必须保持失败可见，不允许用兜底状态掩盖真实异常。
- 前后端状态枚举、详情字段、清理策略必须一起对齐，不能只改后端。

## Risk Assessment

- 数据库 schema 变更会影响 `request_records` / `request_candidates` 的查询与测试夹具。
- 流式与 WebSocket 终态改造会影响实时链路，若处理不完整可能继续留下僵尸记录。
- 前端状态与详情面板改造需要与后端返回字段同步，否则会直接出现渲染错误或类型错误。

## Child Deliverables

- 子任务 1：重构请求记录 schema、枚举与共享类型。
- 子任务 2：把 `request_records` 改成权威主记录写入链路。
- 子任务 3：实现 HTTP 流式与 WS 的终态、取消归因、断开后持久化。
- 子任务 4：补齐 provider/client 双视角 payload 与错误提取。
- 子任务 5：重构清理与保留策略。
- 子任务 6：更新前端状态、详情和 payload 展示。
- 子任务 7：对齐 migration 默认值、种子与配置面板。
- 子任务 8：补齐测试并完成验证。

## Dependency Notes

- 子任务 2 依赖子任务 1 的 schema 与类型。
- 子任务 3 依赖子任务 2 的主记录语义。
- 子任务 4 依赖子任务 1 和子任务 2 的字段落点。
- 子任务 5 依赖子任务 4 的 payload 字段。
- 子任务 6 依赖子任务 1、4 的字段与状态枚举。
- 子任务 7 依赖子任务 1、5、6。
- 子任务 8 依赖全部前序子任务。

## Child Task Types

- `single-full`

## Done-When

- [ ] `SUBTASKS.csv` 的 8 个子任务全部为 `DONE`
- [ ] Rust 测试、前端 lint / build 与关键请求记录链路验证通过
- [ ] 流式与 WS 不再留下 `streaming` 僵尸记录，错误详情不再只有通用文案
