# Single Task Spec

## Goal

- 把 `request_records` 从候选聚合快照改成显式写入的权威主记录。

## Scope

- `apps/hook_backend/src/llm_proxy/audit.rs`
- `crates/storage/src/provider/request_record_repository.rs`
- `crates/storage/src/provider/request_candidate_query.rs`
- `crates/storage/src/provider/request_record_query.rs`
- `crates/storage/src/provider/types.rs`

## Done-When

- 主记录能在候选创建前初始化
- 每次 attempt 更新会显式回写主记录终态与客户端视角字段
- `request_record` 详情不再依赖 candidate 聚合反推主状态
