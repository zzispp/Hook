# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 补齐 usage flush 在冷启动/重启后的幂等恢复语义。
- 覆盖进程崩溃在 DB 事务提交后、Redis processing 清理前的窗口，避免 token/model usage 重复计数。

## Non-Goals

- 不改变钱包结算同步强一致路径。
- 不改变 request audit 历史写入策略。

## Constraints

- 不使用静默 fallback。
- 恢复路径必须显式区分已落库 batch 和未落库 batch。
- DB batch 标记必须和 usage 增量处于同一事务。

## Done-When

- [ ] processing batch 带稳定 batch id。
- [ ] DB usage batch 写入具备 batch id 幂等检查。
- [ ] 重启后已提交 batch 只清理 Redis，不重复加计数。
- [ ] 检查与测试通过。
