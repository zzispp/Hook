# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- 修复性能监控 overview 时间窗口数据缺失问题：`today`、`7d`、`30d`、`all` 需要包含当前窗口内的请求数据，不能只依赖整点/零点落盘快照。
- 核心请求趋势和延迟/TTFT 图应能读取已有 `request_records` 的实时聚合结果。

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- 不调整采集字段语义。
- 不改前端视觉布局。
- 不引入模拟数据或静默 fallback。

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Rust backend/storage。
- 后端单测使用 60 秒超时。
- 失败必须显式暴露。

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024 / Next.js
- **Package manager**: Cargo / pnpm
- **Test framework**: Rust unit tests
- **Build command**: `cargo test -p storage performance_monitoring`
- **Existing test count**: not enumerated

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — local Postgres confirmed.
- [x] Breaking changes to existing code — scoped to monitoring overview aggregation.
- [x] Large file generation — not applicable.
- [x] Long-running tests — timeout configured.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- `crates/storage/src/performance_monitoring/*` overview/query changes.
- Focused Rust tests for range/current-window behavior.
- Progress notes in this task directory.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] `today` 使用最近 24 小时内可用的窗口聚合点，包含当前未落盘窗口。
- [ ] `7d`、`30d`、`all` 包含当前窗口数据。
- [ ] `all` 在没有 day 快照但存在请求记录时不返回空。
- [ ] Rust 单测通过。

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
cargo test -p storage performance_monitoring
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
