# Progress Log

---

## Session Start

- **Date**: 2026-06-16
- **Task name**: `20260616-token-aware-routing-simulation`
- **Task dir**: `.codex-tasks/20260616-token-aware-routing-simulation/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust / Axum + SeaORM / Next.js

---

## Context Recovery Block

- **Current milestone**: complete
- **Current status**: DONE
- **Key context**: `/api/admin/routing/rankings` is now token-aware and uses API token, model, API format, stream flag, cache affinity, scheduler seed, effective profile, and dynamic scoring. `/api/admin/routing/preview` was removed.
- **Known issues**: `just test` is bounded by the repository 60-second wrapper and timed out during full-suite execution, after targeted checks passed and without reporting test failures.
- **Next action**: none

---

## 2026-06-16T14:20:00+08:00

- Implemented backend token-aware rankings request/response shape.
- Added shared `token_context` selection helper for token, group, model, affinity, matching, and scheduler ordering.
- Removed preview handler/export/types/default API seed.
- Validation: `timeout 60 cargo test -p hook_backend routing` passed after incremental compilation.

## 2026-06-16T14:25:00+08:00

- Added focused tests for routing seed validation, effective profile precedence, load-balance seed stability, and token active/expired checks.
- Validation: `timeout 60 cargo test -p hook_backend routing` passed with 25 tests.

## 2026-06-16T14:29:00+08:00

- Updated routing observability UI to select an active admin API token, derive group read-only from token, remove manual profile tabs, and use response profile for summary/editor.
- Removed frontend preview helper and endpoint.
- Updated backend i18n seeds for real path simulation labels.
- Validation: `pnpm lint:frontend` passed.

## 2026-06-16T14:36:06+08:00

- Validation: `timeout 60 cargo test -p storage routing` passed in 59.15s.
- Validation: `just test` first run timed out during compile at `Compiling types v1.0.2`.
- Validation: `just test` second run timed out later during full-suite execution in `user` crate tests after `infra::password::tests::verifies_configured_admin_password_hash ... ok`; no failing test output appeared before the timeout.

## 2026-06-16T14:41:08+08:00

- Validation: reran `just test`. It reached doc tests after passing `user` and `wallet` crate tests, then timed out at `Doc-tests captcha`; no failing test output appeared before the timeout.
