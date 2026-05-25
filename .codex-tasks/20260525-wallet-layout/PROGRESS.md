# Progress

## Recovery

- Task: Optimize wallet center layout.
- Shape: single-full.
- Current: Step 2, refactoring component boundaries.
- Truth: `.codex-tasks/20260525-wallet-layout/TODO.csv`.

## Notes

- 2026-05-25: User approved the design plan.
- 2026-05-25: Existing `wallet-center-view.tsx` contains card-code, recharge, and ledger in one view; split is needed to keep changes small and testable.
- 2026-05-25: Added `WalletDepositSection`, `WalletCardCodePanel`, and `WalletLedgerSection`. The page now reads as summary -> deposit operations -> ledger.
- 2026-05-25: `WalletRechargePanel` now has custom amount and package tabs. Custom amount is explicitly disabled because no payment channel flow exists yet.
- 2026-05-25: Frontend lint passed after import ordering fixes. Continue with function-size cleanup and final validation.
- 2026-05-25: Extracted `useWalletDepositActions` to keep the page component focused on composition.
- 2026-05-25: Final validation passed: i18n JSON parse, frontend lint, frontend production build, backend i18n seed tests.
- 2026-05-25: Browser opened `http://localhost:8082/dashboard/wallet/`; route title loaded, but the current browser user is blocked by `Permission denied`, so wallet-body visual verification was not possible in that session.

## Final Validation

- `node -e "JSON.parse(...admin.cn.json); JSON.parse(...admin.en.json)"` passed.
- `pnpm --filter hook_frontend lint` passed.
- `pnpm --filter hook_frontend build` passed.
- `cargo test -p backend migration::baseline::seed_domain::tests -- --nocapture` passed.
- Browser route check reached `/dashboard/wallet/` with title `钱包中心 | Dashboard - Hook`, then showed `Permission denied`.
