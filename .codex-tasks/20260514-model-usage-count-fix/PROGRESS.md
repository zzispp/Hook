# Progress Log

> Auto-maintained by Taskmaster. Each entry records what happened, why, and what's next.
> This file serves as both decision audit trail and context-recovery anchor.

---

## Session Start

- **Date**: 2026-05-14 18:24
- **Task name**: `20260514-model-usage-count-fix`
- **Task dir**: `.codex-tasks/20260514-model-usage-count-fix/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv` (4 milestones)
- **Environment**: `Rust workspace / pnpm frontend / cargo test-check`

---

## Context Recovery Block

> If you are resuming this task after compaction, session restart, or context loss,
> read this section FIRST to restore working state.

- **Current milestone**: #4 — Run final verification
- **Current status**: DONE
- **Last completed**: #4 — Run final verification
- **Current artifact**: `TODO.csv`
- **Key context**: 已在 `storage::model` 增加 `record_usage`，并在 `llm_proxy` 成功请求审计路径中调用；前端读取口径无需改动。
- **Known issues**: 无新增已知问题。
- **Next action**: None for this task.

## Milestone 1: Inspect model usage count data flow

- **Status**: DONE
- **Completed**: 18:30
- **What was done**:
  - 确认前端模型管理页和模型详情都直接读取 `GlobalModelResponse.usage_count`。
  - 确认 `global_models` 表和对应 record/response 映射都已包含 `usage_count` 字段。
  - 确认 `llm_proxy` 的成功请求审计只会记录 token usage，没有模型 usage 的任何存储写入。
- **Validation**: `cargo test -p storage api_token_usage -- --nocapture` → exit 0
- **Files changed**:
  - `.codex-tasks/20260514-model-usage-count-fix/SPEC.md`
  - `.codex-tasks/20260514-model-usage-count-fix/TODO.csv`
  - `.codex-tasks/20260514-model-usage-count-fix/PROGRESS.md`
- **Next step**: Milestone 2 — Identify root cause and define minimal fix

## Milestone 2: Identify root cause and define minimal fix

- **Status**: DONE
- **Completed**: 18:37
- **What was done**:
  - 确认 `CandidateTrace` 已携带 `global_model_id`，因此代理成功后具备模型 usage 归因信息。
  - 确认 `record_attempt` 当前仅更新 request candidate / request record / token usage，没有模型 usage 更新。
  - 确定最小修复点为 `storage::model` 新增递增接口，并在成功完成的 audit 路径复用同一计数判定。
- **Validation**: `rg -n "usage_count|record_usage|global_model_id" apps/hook_backend crates` → root cause confirmed
- **Files changed**:
  - `.codex-tasks/20260514-model-usage-count-fix/TODO.csv`
  - `.codex-tasks/20260514-model-usage-count-fix/PROGRESS.md`
- **Next step**: Milestone 3 — Implement usage_count persistence and tests

## Milestone 3: Implement usage_count persistence and tests

- **Status**: DONE
- **Completed**: 18:39
- **What was done**:
  - 在 `crates/storage/src/model/repository.rs` 新增 `ModelStore::record_usage`，通过 SQL 自增 `global_models.usage_count`。
  - 在 `crates/storage/src/model/types.rs` 增加 `GlobalModelUsageRecord`，并在 `mod.rs` 导出。
  - 在 `apps/hook_backend/src/llm_proxy/audit.rs` 中把模型 usage 递增接到成功完成请求后的审计路径。
  - 新增 `crates/storage/tests/model_usage.rs` 覆盖递增 SQL 和缺失模型时报错行为。
- **Validation**: `cargo test -p storage model_usage -- --nocapture` → exit 0
- **Files changed**:
  - `apps/hook_backend/src/llm_proxy/audit.rs`
  - `crates/storage/src/model/mod.rs`
  - `crates/storage/src/model/repository.rs`
  - `crates/storage/src/model/types.rs`
  - `crates/storage/tests/model_usage.rs`
- **Next step**: Milestone 4 — Run final verification

## Milestone 4: Run final verification

- **Status**: DONE
- **Completed**: 18:40
- **What was done**:
  - 运行 `cargo fmt --all` 对齐 Rust 风格。
  - 运行后端编译检查，确认 llm_proxy 到 storage 的调用链无编译问题。
- **Validation**: `cargo check -p backend` → exit 0
- **Files changed**:
  - `apps/hook_backend/src/llm_proxy/audit.rs`
  - `crates/storage/src/model/mod.rs`
  - `crates/storage/src/model/repository.rs`
  - `crates/storage/src/model/types.rs`
  - `crates/storage/tests/model_usage.rs`
- **Next step**: Complete

## Final Summary

- **Total milestones**: 4
- **Completed**: 4
- **Failed + recovered**: 0
- **External unblock events**: 0
- **Total retries**: 0
- **Files created**: 4
- **Files modified**: 6
- **Key learnings**:
  - 问题不在前端，而在后端代理审计链路缺少 `global_models.usage_count` 的持久化更新。
  - 现有代码已经具备模型归因所需的 `global_model_id`，因此可以在 audit 成功路径上做最小接入。
