# Progress

## Recovery

- 任务: Add shared username/password constraints with trim.
- 形态: single-full
- 进度: 5/5
- 当前: Completed.
- 文件: `.codex-tasks/20260507-auth-credential-constraints/TODO.csv`
- 下一步: None.

## Log

- Used Brave MCP to check common password and username guidance.
- Selected explicit rules in SPEC.md: username 3-30 ASCII letters/digits/_/-, alphanumeric edges; password 8-128; trim both before validation and use.
- Added backend tests first. They initially failed on trim, invalid username/password constraints, and sign-up without role/status.
- Implemented backend trim before validation, persistence/hash, and login verification; sign-up/update email is trimmed as well.
- Moved credential length constants to `crates/constants/src/auth.rs`.
- Split public sign-up payload to `{ username, email, password }`; backend sets `role = "user"` and `status = "enabled"`.
- Updated frontend sign-in/sign-up schemas to use the same username/password constraints and trim payload values before submit.
- Validation passed: `just test`, `cargo fmt --all --check`, `cargo check`, `cargo clippy -p user --all-targets -- -D warnings`, `cargo clippy -p backend --all-targets -- -D warnings`, `pnpm --dir apps/hook_frontend lint`, `pnpm --dir apps/hook_frontend build`.
