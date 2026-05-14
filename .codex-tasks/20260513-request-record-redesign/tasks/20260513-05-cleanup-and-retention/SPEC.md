# Single Task Spec

## Goal

- 让清理与保留策略覆盖新的主记录 payload 字段与终态语义。

## Scope

- `crates/storage/src/provider/request_record_cleanup.rs`
- `apps/hook_backend/src/request_record_cleanup.rs`
- 相关设置读取与统计输出

## Done-When

- payload 清理同时处理 request_records 与 request_candidates
- 新字段不会绕开 retention 策略
- 清理日志统计仍然可读
