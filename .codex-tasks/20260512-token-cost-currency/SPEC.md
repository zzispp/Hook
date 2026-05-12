# Token Cost Currency Display

## Goal

Make the token management cost column follow the system display currency and USD/CNY exchange rate, matching the request records billing column behavior.

## Scope

- Frontend token management views under `apps/hook_frontend/src/sections/api-tokens`.
- Backend i18n seed keys for the column label.

## Evidence

- Request records use `CurrencyDisplay`, `useSystemSettings`, `useUsdCnyExchangeRate`, and `formatCost`.
- Token management currently renders `row.used_quota` through a local fixed numeric formatter and labels the column with `fields.costCny`.

## Validation

- Run focused frontend lint/build checks when feasible.
