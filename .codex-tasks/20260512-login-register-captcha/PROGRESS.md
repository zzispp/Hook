# Progress Log

---

## Session Start

- **Date**: 2026-05-12 23:50 CST
- **Task name**: `20260512-login-register-captcha`
- **Task dir**: `.codex-tasks/20260512-login-register-captcha/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (7 milestones)
- **Environment**: Rust workspace + Next.js / pnpm / cargo tests

---

## Context Recovery Block

- **Current milestone**: #4 — Wire auth flows to captcha setting
- **Current status**: IN_PROGRESS
- **Last completed**: #3 — Implement backend captcha service and endpoints
- **Current artifact**: `.codex-tasks/20260512-login-register-captcha/TODO.csv`
- **Key context**: Rust captcha service and public routes are added. Auth handlers now call `verify_login_register`, but user route tests still need validation.
- **Known issues**: `just` is not installed in this environment; `apps/hook_frontend/next-env.d.ts` had pre-existing user modifications.
- **Next action**: Run user auth tests with a 60-second timeout and fix any auth integration regressions.

---

## Milestone 1: Clone and inspect Cap source

- **Status**: DONE
- **Started**: 23:50
- **Completed**: 00:01
- **What was done**:
  - Cloned `https://github.com/tiagozip/cap` into `.codex-tasks/20260512-login-register-captcha/raw/cap`.
  - Read Cap agent guidance, core challenge generation/validation, demo server endpoints, and widget API.
- **Key decisions**:
  - Decision: Implement Rust endpoints compatible with Cap widget format-1 PoW challenge/redeem.
  - Reasoning: The widget supports this protocol directly and instrumentation is optional in Cap core; this keeps the login/register feature scoped and testable.
- **Problems encountered**:
  - Problem: None.
  - Resolution: Not applicable.
  - Retry count: 0
- **Validation**: `test -d .codex-tasks/20260512-login-register-captcha/raw/cap` -> exit 0
- **Files changed**:
  - `.codex-tasks/20260512-login-register-captcha/` — task tracking files.
- **Next step**: Milestone 2 — Map existing auth settings and i18n architecture

---

## Milestone 2: Map existing auth settings and i18n architecture

- **Status**: DONE
- **Started**: 00:01
- **Completed**: 00:03
- **What was done**:
  - Located auth payloads and handlers in `crates/types/src/user/api.rs` and `crates/user/src/api/handlers.rs`.
  - Located system settings storage, validation, migration seed, and frontend form files.
  - Confirmed admin settings copy comes from `apps/hook_backend/src/migration/defaults/i18n/admin.*.json`.
- **Key decisions**:
  - Decision: Add a public captcha config endpoint because login/register pages cannot read admin settings.
  - Reasoning: The feature switch must control auth pages before authentication.
- **Problems encountered**:
  - Problem: Search output is large.
  - Resolution: Followed with targeted file reads for actual edit sites.
  - Retry count: 0
- **Validation**: `rg -n "login|register|system|i18n" apps crates config -S` -> exit 0
- **Files changed**:
  - `.codex-tasks/20260512-login-register-captcha/` — task tracking updates.
- **Next step**: Milestone 3 — Implement backend captcha service and endpoints

---

## Milestone 3: Implement backend captcha service and endpoints

- **Status**: DONE
- **Started**: 00:03
- **Completed**: 00:14
- **What was done**:
  - Added `crates/captcha` with Cap-compatible format-1 PoW challenge/redeem behavior.
  - Added Redis-backed one-time challenge consumption and one-time redeemed-token consumption.
  - Added `/api/captcha/config`, `/api/captcha/challenge`, and `/api/captcha/redeem` routes.
  - Added the `login_register_captcha_enabled` system setting through backend types, storage, and baseline seed.
- **Key decisions**:
  - Decision: Use Redis state for challenges and redeemed tokens instead of a self-contained signed challenge token.
  - Reasoning: The existing backend already has Redis and this keeps token consumption explicit and one-time.
- **Problems encountered**:
  - Problem: `just check` could not run because `just` is not installed.
  - Resolution: Ran `cargo check -p backend` and `cargo test -p captcha`; both passed.
  - Retry count: 0
- **Validation**: `cargo check -p backend` -> exit 0; `cargo test -p captcha` -> exit 0
- **Files changed**:
  - `crates/captcha/` — new captcha application, API, and Redis infra.
  - `crates/types/src/captcha.rs` — captcha API DTOs.
  - Backend startup/config/settings storage/migration files — route wiring and setting persistence.
- **Next step**: Milestone 4 — Wire auth flows to captcha setting

---
