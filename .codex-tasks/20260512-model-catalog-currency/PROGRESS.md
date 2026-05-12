# Progress

## Recovery

- 任务: 修复模型目录价格未跟随后台 CNY 设置的问题
- 形态: single-full
- 进度: 3/3
- 当前: Complete
- 文件: `.codex-tasks/20260512-model-catalog-currency/TODO.csv`
- 下一步: None.

## Log

- 2026-05-12: Created task record and started source inspection.
- 2026-05-12: Root cause confirmed. User model catalog formats prices with literal `$`; admin model management reads system settings and exchange rate. The user model catalog route does not have a non-admin display currency source.
- 2026-05-12: Implemented `/api/settings/display-currency` as an authenticated base API. Frontend model catalog now uses the shared currency display formatter across table, mobile cards, detail drawer, tiered pricing, request pricing, and group pricing.
- 2026-05-12: Validation passed: `cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `just test`, and `git diff --check`.
