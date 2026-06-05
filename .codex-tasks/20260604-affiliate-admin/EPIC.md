# 返佣管理后台

## Goal

新增独立后台返佣管理菜单，提供概览、邀请关系管理、返佣记录审计、报表和 CSV 导出。

## Boundary

- 管理员可改绑或清空邀请关系。
- 关系变更只影响未来充值返佣。
- 历史返佣记录、钱包流水、已入账 gift_balance 不重算、不冲正、不删除。
- 后台文案写入后端 i18n seed JSON，不新增前端 admin locale JSON。

## Validation

- Backend: focused tests, `just check`.
- Frontend: `pnpm lint:frontend`, `pnpm build:frontend`.
