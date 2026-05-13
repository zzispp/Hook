# Request Record Full Filters

## Goal

请求记录列表筛选必须按数据库中的全部请求记录生效，而不是只在最近候选记录窗口内筛选。管理端需要新增模型、API 格式、提供商、类型筛选。

## Scope

- 后端 `GET /api/admin/request-records` 增加筛选参数：
  - `model_id`
  - `provider_id`
  - `api_format`
  - `type`
- `type` 使用请求记录表格里的类型语义：`stream` / `non_stream`。
- `api_format` 同时匹配 client API format 与 provider API format。
- 保持 `status` 使用聚合请求状态语义。
- 前端请求记录页新增对应筛选控件。
- 更新 admin i18n seed。

## Non-goals

- 不新增请求记录物化表。
- 不改请求记录的写入链路。
- 不添加 mock 或静默 fallback。
