# 用户侧返佣中心

## Goal

新增用户侧独立返佣中心，展示邀请码、邀请统计、下级脱敏列表、个人返佣记录和 CSV 导出，并从钱包中心移除返佣卡片。

## Boundary

- 用户侧只读，不允许改绑/清空邀请关系。
- 查询强制限定当前用户为邀请人。
- 下级邮箱只返回脱敏值。
- 不复用管理员 `/api/admin/affiliates/*` 作为普通用户入口。
- 后台文案只更新后端 i18n seed JSON。

## Validation

- Backend: focused user/storage/routes/defaults tests, cargo check.
- Frontend: pnpm lint/build.
