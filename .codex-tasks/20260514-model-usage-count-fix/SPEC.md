# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape

- **Shape**: `single-full`

## Goals

- 排查并修复“模型管理”页面中的模型 `usage_count` 不会随着用户调用模型而增加的问题。
- 让修复符合现有 Rust monorepo 分层、存储仓库模式、测试方式和 admin i18n 规范。

## Non-Goals

- 不改动“调用次数”的展示文案和前端交互。
- 不引入兼容兜底、模拟计数或新的业务口径。

## Constraints

- 遵守现有 `apps/hook_backend` 与 `crates/*` 的分层边界。
- 优先复用现有存储仓库、代理审计与测试模式。
- 对计数语义的修改必须先定位读写链路，再做最小修复。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: `Rust workspace + Next.js/pnpm`
- **Package manager**: `pnpm`
- **Test framework**: `cargo test` / `cargo check` / frontend lint-build checks
- **Build command**: `just build` / `pnpm build:frontend`
- **Existing test count**: `待按相关 crate 实测`

## Risk Assessment

- [x] External dependencies (APIs, services) — availability confirmed?
- [x] Breaking changes to existing code — impact assessed?
- [x] Large file generation — disk space sufficient?
- [x] Long-running tests — timeout configured?

## Deliverables

- 模型调用次数不递增的根因说明。
- 对应后端修复代码与必要测试。
- 本地验证记录。

## Done-When

- [ ] 用户成功调用模型后，后端存在明确的 `global_models.usage_count` 递增写入路径。
- [ ] 相关测试或检查能覆盖本次修复的关键行为。
- [ ] 现有 admin 模型列表读取逻辑无需额外兼容即可拿到更新后的调用次数。

## Final Validation Command

```bash
cargo test -p storage model_usage -- --nocapture && cargo check -p backend
```

## Demo Flow (optional)

1. 发起一次经过 `llm_proxy` 的模型请求。
2. 请求落到某个 `global_model_id`。
3. 管理端读取模型列表时，`usage_count` 比调用前增加。
