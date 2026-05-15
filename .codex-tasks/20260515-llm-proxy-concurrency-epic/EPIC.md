# Epic Specification

## Goal

- 将 `llm_proxy` 从“同步 DB 审计 + 同步 DB 计费 + 易失效认证缓存”的热路径，调整为以 Redis runtime snapshot / cache 为主、DB 为控制面与最终落库面的结构。

## Non-Goals

- 不在本 Epic 内引入新外部基础设施。
- 不改动前端管理界面交互。
- 不做与代理热路径无关的通用重构。

## Constraints

- 必须遵守现有 Rust/Axum/SeaORM 分层，组合根保留在 `apps/hook_backend`，业务规则保留在共享 crates。
- 不引入静默 fallback；失败必须显式暴露。
- 后端改动遵守 TDD，先补失败测试再改生产代码。
- 不破坏现有 API 行为与请求记录查询接口的数据契约，除非在子任务中显式迁移。

## Risk Assessment

- 热路径计费与审计改造会触及请求完成语义，存在结算与记录不一致风险。
- 异步审计需要明确失败暴露策略，不能变成“看起来成功但数据丢失”。
- 钱包并发改造可能触及数据库事务语义，需要验证现有存储层支持方式。

## Child Deliverables

- 阶段 1：认证缓存与 token 用量同步解耦
- 阶段 1：request record policy 并入 runtime snapshot
- 阶段 1：钱包结算并发正确性修复
- 阶段 2：审计写入事件化与异步落库
- 阶段 2：活跃请求态与 runtime snapshot 进一步去 DB 化

## Dependency Notes

- 子任务 2 依赖子任务 1 的 runtime cache 方向确认。
- 子任务 4 依赖子任务 2 的 runtime snapshot 基础与子任务 3 的结算边界。
- 子任务 5 依赖子任务 4 的事件流与 runtime state 结构。

## Child Task Types

- `single-full`

## Done-When

- [ ] `SUBTASKS.csv` 所有子任务完成或状态明确
- [ ] 阶段 1 代码落地并通过验证
- [ ] 阶段 2 至少完成骨架实现与可验证路径
- [ ] 后端回归验证通过

