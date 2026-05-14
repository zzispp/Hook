# Single Task Spec

## Goal

- 为请求记录重构建立新的底层字段与状态语义，覆盖数据库基线、storage entity、共享类型与前端状态枚举。

## Scope

- `apps/hook_backend/src/migration/baseline/**`
- `crates/storage/src/provider/**`
- `crates/types/src/provider/**`
- `apps/hook_frontend/src/types/provider.ts`
- 请求记录相关前端状态工具文件

## Done-When

- `request_records` / `request_candidates` 拥有新字段与状态枚举
- 前后端类型能表达 `cancelled` 与双视角 payload 字段
- 相关编译或单测能够开始通过
