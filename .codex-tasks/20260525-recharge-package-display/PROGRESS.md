# Progress

## Recovery

- Task: Fix recharge package display and ratio warning.
- Shape: single-full.
- Current: Step 1, tracing source and request chain.
- Truth: `.codex-tasks/20260525-recharge-package-display/TODO.csv`.

## Notes

- 2026-05-25: Started from user request. Initial search found recharge user strings in backend i18n seed and recharge frontend sections/actions.
- 2026-05-25: Root cause found: `useUserRechargePackages()` and `useUserRechargeOrders()` defaulted to `page = 1`, but `pageQuery()` adds 1 before sending to the backend. User package requests were hitting backend page 2.
- 2026-05-25: Removed user-facing pending-order copy from i18n seeds and replaced the combined info notice with a standalone warning for `wallet.recharge.ratio`.
- 2026-05-25: Validation so far: old text/key search has no results, i18n JSON parse passed, frontend lint passed.
- 2026-05-25: Backend migration seed tests passed for i18n flatten/default translation coverage.

## Final Validation

- `rg -n "wallet\\.recharge\\.description|购买套餐会创建待支付订单|Balance is not credited until payment is integrated|Buying a package creates a pending order" apps crates` returned no matches.
- `node -e "JSON.parse(...admin.cn.json); JSON.parse(...admin.en.json)"` passed.
- `pnpm --filter hook_frontend lint` passed.
- `cargo test -p backend migration::baseline::seed_domain::tests -- --nocapture` passed.
