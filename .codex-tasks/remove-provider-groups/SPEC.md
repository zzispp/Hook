# Remove Provider Groups

## Goal

彻底移除提供商分组能力，只保留提供商密钥分组作为计费分组的上游 key 访问限制维度。

## Scope

- 删除 provider group API、类型、存储、迁移和前端入口。
- 删除 billing group 的 provider group 绑定字段。
- 运行时只按 provider key group 限制；无 key group 表示不限制。
- 不做兼容 fallback。

## Validation

- `just check`
- `just test`
- `pnpm lint:frontend`
- `pnpm build:frontend`
- 搜索确认 provider group 残留只出现在明确允许的删除迁移中。
