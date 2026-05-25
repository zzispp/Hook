# Progress

## Recovery

- 任务: 用户侧充值入口和购买套餐链路
- 形态: single-full
- 当前: Complete
- 下一步: 汇总变更和验证结果。

## Notes

- 用户钱包页已有 `/dashboard/wallet` 和 `wallet_center` 菜单权限，适合直接新增充值 tab。
- 当前充值 domain 只有 admin 套餐 CRUD、admin 订单列表和支付渠道管理。
- 用户创建套餐订单需要读取 `SystemSettings` 的充值开关、比例、过期时间和 min/max。
- 已新增 `/api/recharge-packages`、`/api/recharge-orders` GET/POST 处理器与 service/storage 方法。
- `timeout 60s cargo test -q -p recharge` 已通过 12 个测试。
- 已新增用户侧充值 API 默认定义并绑定 `wallet_center`；`just check` 通过。
- 钱包中心已新增充值 tab 和套餐购买面板；`pnpm lint:frontend`、`pnpm build:frontend` 通过。
- 最终验证通过：`just check`、`timeout 60s cargo test -q -p recharge`、`pnpm lint:frontend`、`pnpm build:frontend`。
