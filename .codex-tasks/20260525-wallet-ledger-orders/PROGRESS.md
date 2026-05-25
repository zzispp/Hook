# Progress

## Recovery

- Task: Move recharge orders into ledger card tabs and adjust spacing.
- Shape: single-full.
- Current: Done.
- Truth: `.codex-tasks/20260525-wallet-ledger-orders/TODO.csv`.

## Validation

- 2026-05-25 22:11:12 CST: `pnpm --filter hook_frontend lint` passed.
- 2026-05-25 22:11:12 CST: `pnpm --filter hook_frontend build` passed.

## Result

- Recharge orders render in the wallet ledger card under the `充值订单` tab.
- The recharge card no longer receives or renders recharge orders.
- The ledger card header uses extra vertical spacing between the title, tabs, and toolbar content.
