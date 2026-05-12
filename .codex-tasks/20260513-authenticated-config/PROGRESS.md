# Progress

## Recovery

- 任务: 将已登录基础 API 从常量迁到配置
- 形态: single-full
- 进度: 3/3
- 当前: Complete
- 文件: `.codex-tasks/20260513-authenticated-config/TODO.csv`
- 下一步: None.

## Log

- 2026-05-13: Confirmed `auth.whitelist` bypasses login, while authenticated base APIs are currently hard-coded in startup.
- 2026-05-13: Added `auth.authenticated` to the config schema and YAML, then wired startup authorization setup to read it.
- 2026-05-13: Validation passed: `cargo check -p backend`, `cargo test -p configuration`, `just test`, and `git diff --check`.
