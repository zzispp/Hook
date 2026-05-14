# Single Task Spec

## Goal

- 为这次请求记录重构补齐关键测试与最终验证。

## Scope

- `crates/storage/tests/**`
- `apps/hook_backend` 相关单测
- `apps/hook_frontend` lint/build 检查

## Done-When

- storage 关键测试覆盖主记录显式写入
- llm_proxy 终态路径有最小验证
- `just test` 和 `pnpm lint:frontend` 达到可接受状态
