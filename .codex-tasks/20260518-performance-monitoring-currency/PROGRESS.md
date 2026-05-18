# Progress

## 2026-05-18

- Confirmed performance monitoring used `fCurrency`, which formats by locale default currency.
- Confirmed request records and model pricing use backend system currency setting through `CurrencyDisplay`.
- Implementing the same display currency path in performance monitoring.
- Wired performance monitoring summary cost through `/api/settings/display-currency`.
- Replaced performance monitoring cost formatting with `formatMoneyCompact`.
- Validation passed: `pnpm lint:frontend`, `pnpm build:frontend`.
- `timeout` command is not available in this macOS shell; first validation attempt failed before running lint.
