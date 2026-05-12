# Progress

## 2026-05-12

- Confirmed request records derive display currency from `useSystemSettings()` and fetch USD/CNY only when the selected currency is CNY.
- Confirmed token management cost column uses `formatCurrency(row.used_quota)` and the fixed `fields.costCny` label.
- Added admin-scoped token currency display state that uses system currency and USD/CNY exchange-rate data.
- Updated the token table to format cost through the shared money formatter when currency display is available and to render a dynamic `费用(CNY/USD)` style header.
- Validation passed: changed-file eslint, full `pnpm lint:frontend`, `pnpm --filter hook_frontend exec tsc --noEmit`, and `pnpm build:frontend`.
