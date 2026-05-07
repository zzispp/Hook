# Progress

## Recovery

- 任务: 新增 axum + toasty 用户管理 API
- 形态: single-full
- 进度: 12/12
- 当前: Complete
- 文件: `.codex-tasks/20260507-user-management/TODO.csv`

## Notes

- Existing backend is a single empty binary crate under `apps/hook_backend`.
- User specified Axum and Toasty ORM.
- PostgreSQL target: `postgres://postgres:123456@localhost:5433/postgres`.
- User clarified crate layout: crates must be split by module; module crate owns `domain/application/infra/api`; constants and config live in dedicated crates; crate package names should not start with `hook_`.
- Implemented workspace packages: `backend`, `configuration`, `constants`, `types`, `user`.
- Extracted shared user domain and API transport types into `crates/types`, following the reference project's separate primitives-style type crate approach.
- User requested bringing forward three reference-project patterns: standalone `storage` crate, unified API response envelope, and root `justfile`/`rustfmt.toml`.
- Extracted Toasty persistence into `crates/storage`; `cargo check` passed.
- Added shared API response structs under `crates/types`; `cargo test -p types` passed.
- Added root `justfile` and `rustfmt.toml`; `just --list` passed.
- Final verification passed: `cargo fmt --all --check`, `cargo check`, and `just test`.
- User clarified the response envelope must follow `/Users/bubu/ZwjProjects/new-api` because Hook is intended as an AI unified gateway.
- `new-api` common response shape is `success/message/data` for success and `success/message` for errors.
- Updated `types::response` to match that shape and made application API errors return HTTP 200 with `success: false`.
- Response verification passed: `cargo test -p types`, `cargo fmt --all --check`, `cargo check`, and `just test`; added a user API error test for HTTP 200 error envelopes.
- `cargo metadata --no-deps --format-version 1` passed.
- `cargo check -p backend` passed.
- `cargo fmt --all --check` passed.
- `cargo check` passed.
