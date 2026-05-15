# Progress Log

> Auto-maintained by Taskmaster. Each entry records what happened, why, and what's next.
> This file serves as both decision audit trail and context-recovery anchor.

---

## Session Start

- **Date**: 2026-05-15 13:40
- **Task name**: `20260515-v1-models`
- **Task dir**: `.codex-tasks/20260515-v1-models/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (3 milestones)
- **Environment**: `Rust workspace / cargo test`

---

## Context Recovery Block

> If you are resuming this task after compaction, session restart, or context loss,
> read this section FIRST to restore working state.

- **Current milestone**: #4 — 运行最终校验并整理结果
- **Current status**: IN_PROGRESS
- **Last completed**: #4 — 运行最终校验并整理结果
- **Current artifact**: `PROGRESS.md`
- **Key context**: `/v1/models` 已对齐 `new-api` 语义，测试通过，并用用户提供的 token 在本地服务上验证返回 `gpt-5.5`。
- **Known issues**: 无。
- **Next action**: 输出最终结果说明。

---

## Milestone 0: 修正验证命令

- **Status**: DONE
- **Started**: 13:45
- **Completed**: 13:47
- **What was done**:
  - 发现 `cargo test -p hook_backend ...` 无法执行，因为 workspace 中实际包名为 `backend`。
  - 更新了任务文档中的最终校验命令与步骤命令。
- **Key decisions**:
  - Decision: 使用 `-p backend` 作为后续测试目标。
  - Reasoning: `apps/hook_backend/Cargo.toml` 的 `[package].name` 为 `backend`。
  - Alternatives considered: 直接运行 workspace 全量测试，但范围过大，不利于快速迭代。
- **Problems encountered**:
  - Problem: 初始测试命令指向不存在的 package。
  - Resolution: 先核对 Cargo package 名称并修正文档。
  - Retry count: 0
- **Validation**: `rg -n "^name\\s*=\\s*\\\"" Cargo.toml apps/*/Cargo.toml crates/*/Cargo.toml` → exit 0
- **Files changed**:
  - `.codex-tasks/20260515-v1-models/SPEC.md` — 修正最终校验命令
  - `.codex-tasks/20260515-v1-models/TODO.csv` — 修正步骤验证命令
  - `.codex-tasks/20260515-v1-models/PROGRESS.md` — 记录修正过程
- **Next step**: Milestone 1 — 建立任务台账并确认实现切点

## Milestone 1: 建立任务台账并确认实现切点

- **Status**: DONE
- **Started**: 13:40
- **Completed**: 13:47
- **What was done**:
  - 建立了 `taskmaster` 的 `SPEC.md`、`TODO.csv`、`PROGRESS.md`。
  - 确认 `/api/models/catalog` 不适合作为 API 令牌模型列表接口，决定在 `llm_proxy` 下新增 `/v1/models`。
- **Key decisions**:
  - Decision: 将“模型可见性”抽成共享模块，而不是把逻辑继续堆进 `selection.rs`。
  - Reasoning: 需要同时服务真实转发和模型目录接口，且 `selection.rs` 已接近上限。
  - Alternatives considered: 直接在 handler 中复制过滤逻辑，但会造成行为分叉。
- **Problems encountered**:
  - Problem: 初始验证命令的 package 名错误。
  - Resolution: 已在 Milestone 0 修正。
  - Retry count: 0
- **Validation**: `test -f .codex-tasks/20260515-v1-models/SPEC.md && test -f .codex-tasks/20260515-v1-models/PROGRESS.md && test -f .codex-tasks/20260515-v1-models/TODO.csv` → exit 0
- **Files changed**:
  - `.codex-tasks/20260515-v1-models/SPEC.md` — 定义目标与校验
  - `.codex-tasks/20260515-v1-models/PROGRESS.md` — 记录进度与恢复点
  - `.codex-tasks/20260515-v1-models/TODO.csv` — 定义执行步骤
- **Next step**: Milestone 2 — 先补失败测试再实现 `/v1/models` 与共享过滤逻辑

## Milestone 2: 先补失败测试再实现 /v1/models 与共享过滤逻辑

- **Status**: DONE
- **Started**: 13:47
- **Completed**: 14:02
- **What was done**:
  - 新增 `apps/hook_backend/src/llm_proxy/model_access.rs`，抽出 API 令牌模型可见性规则。
  - 让候选调度逻辑复用该共享规则，移除 `selection.rs` 中重复实现。
  - 在 `llm_proxy` 新增 `GET /v1/models` 与 `GET /v1/models/{model}`。
  - 新增模型可见性测试与 OpenAI 响应映射测试。
- **Key decisions**:
  - Decision: 模型目录返回模型名作为 OpenAI `id`，时间戳使用固定兼容值。
  - Reasoning: 与参照实现更一致，且现有快照模型并没有可直接映射的 OpenAI 创建时间。
  - Alternatives considered: 从内部 `created_at` 或模型 ID 推导时间戳，但语义不稳定。
- **Problems encountered**:
  - Problem: 首轮共享模块测试失败，原因是测试里误把令牌 allowed models 写成模型名而非全局模型 ID。
  - Resolution: 修正测试数据，保持系统内部按全局模型 ID 校验。
  - Retry count: 1
- **Validation**: `cargo test -p backend llm_proxy::model_access::tests -- --nocapture` → exit 0
- **Files changed**:
  - `apps/hook_backend/src/llm_proxy/model_access.rs` — 新增共享可见性逻辑与测试
  - `apps/hook_backend/src/llm_proxy/handlers.rs` — 新增 `/v1/models` handlers
  - `apps/hook_backend/src/llm_proxy/mod.rs` — 新增路由与模块声明
  - `apps/hook_backend/src/llm_proxy/candidate/selection.rs` — 复用共享规则
- **Next step**: Milestone 3 — 运行最终校验并整理结果

## Milestone 3: 运行最终校验并整理结果

- **Status**: DONE
- **Started**: 14:02
- **Completed**: 14:04
- **What was done**:
  - 运行 `cargo fmt`。
  - 运行 `cargo test -p backend llm_proxy -- --nocapture` 做目标范围回归。
  - 检查 `git diff --stat` 与 `git status --short`，确认改动集中在预期文件。
- **Key decisions**:
  - Decision: 最终验证仅覆盖 `llm_proxy` 目标范围。
  - Reasoning: 本次改动完全落在该模块，目标测试能覆盖新增接口与共享规则。
  - Alternatives considered: 运行全 workspace 测试，但范围过大、反馈噪音高。
- **Problems encountered**:
  - Problem: 并行跑 cargo test 时遇到编译锁等待。
  - Resolution: 等待锁释放后收敛到目标测试结果，不重复争抢构建锁。
  - Retry count: 0
- **Validation**: `cargo test -p backend llm_proxy -- --nocapture` → exit 0
- **Files changed**:
  - `.codex-tasks/20260515-v1-models/TODO.csv` — 标记全部完成
  - `.codex-tasks/20260515-v1-models/PROGRESS.md` — 记录最终验证
- **Next step**: 无

## Final Summary

- **Total milestones**: 5
- **Completed**: 5
- **Failed + recovered**: 1
- **External unblock events**: 0
- **Total retries**: 1
- **Files created**: 4
- **Files modified**: 6
- **Key learnings**:
  - API 令牌模型权限链路内部统一使用全局模型 ID，而不是模型名。
  - 模型目录接口应直接复用调度侧的访问规则，避免接口展示和真实可转发模型不一致。
- **Recommendations for future tasks**:
  - 将来若补 Anthropic 或 Gemini 的 models 列表接口，也继续复用 `model_access.rs`。

## Milestone 4: 对齐 new-api 的模型列表语义

- **Status**: DONE
- **Started**: 14:08
- **Completed**: 14:15
- **What was done**:
  - 对比 `new-api` 的 `/v1/models` 来源，确认它按分组/令牌限制返回模型，不要求 provider 绑定。
  - 调整 `model_access.rs`，去掉列表语义里对 provider 绑定的过滤。
  - 更新测试，使“无 provider 绑定但权限可见”的模型可出现在列表和单模型查询里。
- **Key decisions**:
  - Decision: 只调整 `/v1/models` 语义，不放松真实转发阶段的 provider candidate 校验。
  - Reasoning: 用户要求对齐 `new-api` 的模型列表展示，但仓库的 Debug-First 规则要求真实请求继续暴露“无可用 provider”错误。
  - Alternatives considered: 同时放松转发阶段校验，但会偏离现有后端执行语义。
- **Problems encountered**:
  - Problem: 首轮断言使用的场景混入了用户模型限制，导致预期错误。
  - Resolution: 核对 `new-api` 后保持独立令牌只受分组/令牌限制，修正测试场景。
  - Retry count: 1
- **Validation**: `cargo test -p backend llm_proxy::model_access::tests -- --nocapture` → exit 0
- **Files changed**:
  - `apps/hook_backend/src/llm_proxy/model_access.rs` — 去掉 provider 绑定过滤，更新测试
- **Next step**: Milestone 5 — 运行最终校验并整理结果

## Milestone 5: 运行最终校验并整理结果

- **Status**: DONE
- **Started**: 14:15
- **Completed**: 14:17
- **What was done**:
  - 运行 `cargo test -p backend llm_proxy -- --nocapture`。
  - 启动本地后端并用用户提供的 token 实测 `/v1/models`。
  - 验证实际返回 `gpt-5.5`，与 `new-api` 语义一致。
- **Key decisions**:
  - Decision: 使用用户提供 token 做一次真实 HTTP 验证。
  - Reasoning: 直接确认行为，避免只靠单测推断。
  - Alternatives considered: 仅报告测试通过，但不能证明本地服务实际返回已改变。
- **Problems encountered**:
  - Problem: 本地 5555 端口一度无服务监听。
  - Resolution: 用 `cargo run -p backend` 启动当前代码后重测。
  - Retry count: 0
- **Validation**: `cargo test -p backend llm_proxy -- --nocapture` → exit 0；`curl http://127.0.0.1:5555/v1/models` → 返回 `gpt-5.5`
- **Files changed**:
  - `.codex-tasks/20260515-v1-models/TODO.csv` — 标记完成
  - `.codex-tasks/20260515-v1-models/PROGRESS.md` — 记录最终验证
- **Next step**: 无
- **Recommendations for future tasks**:
  - 将来若补 Anthropic 或 Gemini 的 models 列表接口，也继续复用 `model_access.rs`。

## Recovery: 合并冲突收口

- **Status**: DONE
- **Started**: 14:20
- **Completed**: 14:24
- **What was done**:
  - 修复 `candidate/selection.rs` 合并后残留的导入与共享函数暴露问题。
  - 让 `candidate/selection/matching.rs` 直接复用 `model_access::provider_allowed`，移除失效的 `ids_allow` 依赖。
  - 补齐 `model_access.rs` 测试快照所需的 `SchedulingSnapshot` 新字段。
  - 重新运行 `cargo test -p backend llm_proxy -- --nocapture`，确认目标范围测试全部通过。
- **Key decisions**:
  - Decision: 保持 `selection` 测试入口不变，通过 `pub(super) use` 暴露共享访问函数。
  - Reasoning: 这样能保留现有测试路径，同时避免把权限逻辑再复制回 `selection`。
  - Alternatives considered: 直接修改测试导入路径到 `model_access`，但会放大本次“解决冲突”的改动面。
- **Problems encountered**:
  - Problem: `selection` 在合并后不再持有 `ids_allow`，但 `matching.rs` 还在从它导入。
  - Resolution: 改为直接复用共享权限函数，并清理无用导入。
  - Retry count: 0
- **Validation**: `cargo test -p backend llm_proxy -- --nocapture` → exit 0
- **Files changed**:
  - `apps/hook_backend/src/llm_proxy/candidate/selection.rs` — 清理导入并重新导出共享访问函数
  - `apps/hook_backend/src/llm_proxy/candidate/selection/matching.rs` — 复用共享 provider 过滤逻辑
  - `apps/hook_backend/src/llm_proxy/model_access.rs` — 补齐测试快照字段
- **Next step**: 将冲突文件标记为已解决
