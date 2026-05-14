# Single Task Spec

## Goal

- 让 migration 基线、默认配置种子和管理面板文案与新请求记录行为一致。

## Scope

- `apps/hook_backend/src/migration/baseline/**`
- `apps/hook_backend/src/migration/defaults/**`
- 设置相关前后端面板

## Done-When

- 新字段和新文案可从 baseline 全量落库
- 默认设置不会误导请求记录新行为
- 相关检查可通过
