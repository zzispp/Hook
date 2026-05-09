# Progress Log

## 2026-05-09

- Read aether admin wallet management implementation.
- Confirmed requested feature maps to aether's global `ledger` tab and `/api/admin/wallets/ledger` endpoint.
- Confirmed Hook currently has admin wallet list and per-wallet transactions dialog, but no global admin ledger endpoint or tab.

## Context Recovery Block

- **Current milestone**: #4 — Run validation
- **Current status**: DONE
- **Last completed**: #4 — Run validation
- **Current artifact**: `TODO.csv`
- **Key context**: Need add global ledger endpoint with owner metadata and then front-end tab.
- **Known issues**: Worktree contains unrelated auth/session changes; do not stage or modify them unless required.
- **Next action**: None.

## 2026-05-09 Completion

- Added Hook global admin wallet ledger endpoint at `GET /api/admin/wallets/ledger`.
- Added owner metadata to global ledger responses and shared wallet ledger UI rendering.
- Added admin wallet tabs with `资金流水` first and `钱包列表` second.
- Added global ledger filters for category, reason, and owner type, plus a refresh button in the ledger toolbar.
- Synced baseline RBAC with `admin_wallet_ledger_read` and bound it to `admin_wallets`.
- Inserted the same API/menu binding into the local PostgreSQL DB and ran `cargo run -q -p backend -- --config config/config.yaml migration up` to rebuild RBAC cache.
- Validation passed:
  - `pnpm --filter hook_frontend lint`
  - `pnpm --filter hook_frontend build`
  - `cargo check -q`
  - `perl ... 60 cargo test -q -p wallet admin_ledger`
