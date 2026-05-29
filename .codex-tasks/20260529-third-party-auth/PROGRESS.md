# Progress Log

## Context Recovery Block

- **Current milestone**: complete
- **Current status**: DONE
- **Last completed**: #5 — Run final validation and fix surfaced failures
- **Current artifact**: `TODO.csv`
- **Key context**: Third-party quick login, Provider binding, wallet email OTP binding, account profile, and admin Provider visibility are implemented across Rust backend and Next frontend.
- **Known issues**: `timeout 60 just test` still fails in unrelated `apps/hook_backend/src/llm_proxy/formats.rs::streaming_requests_do_not_route_to_force_non_stream_formats`; that file was not changed by this task. Focused affected package tests pass.
- **Next action**: none for this task.

## Completed Work

- Added optional local password support with `password_set` and explicit passwordless login error.
- Added `user_identities` storage, identity summaries, Provider settings, OAuth/Wallet tickets, Redis-backed ticket/code storage, and startup wiring.
- Implemented OAuth start/callback/bind-existing and Wallet nonce/sign-in/email-code/complete flows.
- Implemented current account profile APIs, password change via email OTP, identity list/unlink, and admin user identity unlink.
- Updated config whitelist/authenticated route policy and default RBAC/i18n seeds.
- Updated frontend auth config/types/actions, login social buttons, OAuth callback page, wallet binding UI, profile page, account drawer entry, settings Provider fields, admin user Provider badges, and admin identity details.
- Split newly large `social_auth` backend implementation and login/settings frontend components to keep new files under 300 lines.

## Validation

- `timeout 60 just check` -> pass.
- `pnpm lint:frontend` -> pass.
- `pnpm build:frontend` -> pass.
- `timeout 60 cargo test -p user social_auth_tests -- --nocapture` -> pass, 9 tests.
- `timeout 60 cargo test -p user` -> pass, 57 tests.
- `timeout 60 cargo test -p setting` -> pass, 38 tests.
- `timeout 60 cargo test -p storage` -> pass.
- `timeout 60 cargo test -p recharge` -> pass.
- `timeout 60 cargo test -p backend proxy_cache_hooks --bin backend` -> pass.
- `timeout 60 cargo test -p backend migration::defaults --bin backend` -> pass.
- `timeout 60 just test` -> fails only at `llm_proxy::formats::tests::streaming_requests_do_not_route_to_force_non_stream_formats`, an unrelated unmodified file.
