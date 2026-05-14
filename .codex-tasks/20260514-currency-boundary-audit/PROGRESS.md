# Progress Log

---

## Session Start

- **Date**: 2026-05-14 15:55 CST
- **Task name**: `20260514-currency-boundary-audit`
- **Task dir**: `.codex-tasks/20260514-currency-boundary-audit/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust workspace / Next.js / cargo + pnpm

---

## Context Recovery Block

- **Current milestone**: completed
- **Current status**: DONE
- **Last completed**: #5 — Run full validation
- **Current artifact**: `TODO.csv`
- **Key context**: Added `crates/currency` with explicit USD/CNY conversion and constants. Request/model/token money remains USD accounting. Wallet/card-code money remains wallet or card-code snapshot currency. System currency is display/input context only and must not silently rewrite stored accounting values.
- **Known issues**: `pnpm build:frontend` prints the existing `Axios error: unauthorized` during static generation and exits 0.
- **Next action**: Final response with summary and validation.

---

## Monetary Domain Audit

- Request billing: `crates/provider/src/application/billing.rs` calculates `token_cost`, `base_cost`, and `total_cost` in `currency::ACCOUNTING_CURRENCY` (`USD`). `apps/hook_backend/src/llm_proxy/audit.rs` writes `cost_currency`, request record costs, and token usage from that billing result. This is accounting-layer money.
- Request records UI: admin request records use system display currency only through `formatMoney`; CNY requires USD/CNY rate and surfaces an error when unavailable. No storage mutation is tied to display currency.
- API tokens: `quota_limit` and `used_quota` are USD accounting counters. Fixed ordinary token list fallback from misleading CNY to USD/display currency, and marked quota input as USD.
- Model/provider prices: `default_tiered_pricing`, `default_price_per_request`, provider effective prices, and billing-group effective prices are USD accounting inputs. Fixed create/edit price labels to show USD.
- Billing groups: `billing_multiplier` is dimensionless and applies to USD accounting price during request billing. No currency conversion needed.
- Wallet balances and wallet transactions: wallet has its own `currency`; balances and ledger amounts are wallet-currency values. Fixed global wallet ledger response to include currency instead of forcing CNY in the frontend, and fixed admin/user wallet amount labels to use the wallet currency.
- User management wallet summary: user list wallet summary previously omitted currency and frontend hardcoded `¥`. Fixed response/type/display to use wallet currency.
- Default user grant: stored setting credits the default CNY wallet path; UI now labels it as wallet-default currency instead of leaving the amount ambiguous.
- Card codes: previous implementation already snapshots card-code generation currency and converts redemption into current target/wallet currency. Refactored conversion into `crates/currency` so future currency expansion has one backend conversion boundary.

## Validation Log

- `cargo test -q -p currency`: passed.
- `cargo test -q -p storage card_code::redemption_currency`: passed; initial dead-code warnings were removed by scoping test-only constants under `cfg(test)`.
- `cargo test -q -p wallet`: passed.
- `cargo test -q -p card_code`: passed.
- `cargo check -q`: passed.
- `pnpm lint:frontend`: passed after import-order fixes.
- `pnpm build:frontend`: passed; printed existing `Axios error: unauthorized` but exited 0.
- `git diff --check`: passed.
