# Progress

## Recovery

- 任务: Implement backend access/refresh token auth and wire frontend.
- 形态: single-full
- 进度: 6/6
- 当前: Completed.
- 文件: `.codex-tasks/20260507-dual-token-auth/TODO.csv`
- 下一步: None.

## Log

- Created task tracking artifacts.
- Mapped backend auth flow across `apps/hook_backend`, `crates/user`, `crates/storage`, `crates/types`, `crates/config`, and frontend JWT client.
- Contract:
  - `POST /api/auth/sign-in` accepts `{ "identifier": string, "password": string }`; `identifier` may be username or email.
  - `POST /api/auth/sign-up` accepts the existing backend user payload `{ "username", "email", "password", "role", "status" }`.
  - Sign-in and sign-up return `{ success, message, data: { user, accessToken, refreshToken } }`.
  - `POST /api/auth/refresh` accepts `{ "refreshToken": string }` and returns `{ success, message, data: { accessToken, refreshToken } }`.
  - `GET /api/auth/me` requires `Authorization: Bearer <accessToken>` and returns `{ success, message, data: { user } }`.
  - JWT TTL config lives in `config/config.yaml` under `jwt.access_token_ttl_seconds` and `jwt.refresh_token_ttl_seconds`; the signing secret is read from `jwt.secret_env`.
- Backend validation completed:
  - `cargo test -p user` passed with 17 tests.
  - `just test` passed for the Rust workspace within the 60-second wrapper.
  - `cargo fmt --all --check`, `cargo check`, `cargo clippy -p backend --all-targets -- -D warnings`, and `cargo clippy -p user --all-targets -- -D warnings` passed.
- Frontend integration completed:
  - Login uses a single username-or-email field.
  - Sign-up uses a single username field plus email/password; email has a placeholder.
  - The JWT session stores access and refresh tokens, refreshes expired access tokens, and calls `/api/auth/me`.
  - `pnpm --dir apps/hook_frontend lint` and `pnpm --dir apps/hook_frontend build` passed.
