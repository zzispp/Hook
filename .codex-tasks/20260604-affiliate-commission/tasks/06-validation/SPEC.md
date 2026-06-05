# 全链路验证与收尾

## Scope

- Run Rust formatting and checks for the changed workspace.
- Run focused backend tests covering setting validation, wallet registration, affiliate summary, and recharge settlement.
- Run frontend lint and production build.
- Record the repository-level `just test` timeout result without masking it.

## Acceptance

- Rust/前端检查通过。
- 关键测试覆盖完成。
- 任务状态和验证结果记录完整。
