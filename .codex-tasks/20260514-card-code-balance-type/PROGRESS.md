# Progress

## Recovery

- 任务: 调整卡密类型到账余额类型
- 形态: single-full
- 进度: 4/4
- 当前: 完成
- 文件: .codex-tasks/20260514-card-code-balance-type/TODO.csv

## Log

- 2026-05-14: 继承上一轮中间状态，`crates/types/src/card_code.rs` 已开始改为 `balance_type` 和生成单 `amount`，其余调用点仍待同步。
- 2026-05-14: 搜索确认卡密类型默认金额字段残留在 service、validation、storage、baseline migration、frontend types/dialog/type table；具体卡密与兑换流水仍需要保留 `recharge_amount` / `gift_amount`。
- 2026-05-14: 后端已改为类型 `balance_type` + 生成 `amount` 契约，具体卡密金额由 service 按类型拆分；`cargo check -q` 通过。
- 2026-05-14: 前端类型弹窗移除金额输入并新增到账类型下拉；生成弹窗改为单金额输入且 label 随类型变化；`pnpm lint:frontend` 通过。
- 2026-05-14: i18n seed 已补 `balanceType`、`rechargeAmount`、`giftAmount`；验证通过：`cargo check -q`、`pnpm lint:frontend`、`pnpm build:frontend`。前端构建期间输出既有 `Axios error: unauthorized`，但命令退出码为 0。
