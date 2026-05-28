# Progress

## 2026-05-28

- Started Full Single task tracking for the dashboard update.
- Inspected Aether dashboard reference from `/tmp/aether-dashboard-ref` at commit `14ad6e9b`.
- Mapped Aether dashboard groups to Hook data: today KPI cards, monthly system/cache/cost summary, daily usage/model/provider charts, daily stats table, and period totals.
- Implemented Hook backend overview response additions without schema changes: selected summary, today summary, monthly summary, and daily stats all aggregate from `request_records`.
- Added `token_count` as the current-window distinct `token_id` count for the user-facing API key KPI.
- Added frontend dashboard components for monthly summary, daily totals, daily charts, and daily table; wired them into the existing overview API.
- Updated backend-controlled admin i18n seed JSON for the new dashboard keys.
- Validation passed: `cargo fmt --all`, `pnpm lint:frontend`, `pnpm build:frontend`, `timeout 60s cargo test -p storage dashboard`, `timeout 60s cargo test -p dashboard`, `jq empty` for admin seed JSON, and `git diff --check`.
- Full `just test` was run twice. The first run hit the repository 60-second wrapper during backend compilation; the second completed enough to fail on an existing backend test outside this change: `apps/hook_backend/src/llm_proxy/formats.rs:461`, assertion `super::formats_compatible("openai:chat", "openai:compact", false)`.
- Added a refinement step after user feedback: period summary cards and period totals must follow the selected `preset` in both data and labels, while the top KPI cards remain today's summary.
- Reworked the frontend period summary to read `overview.summary` instead of `overview.monthly`, and renamed the component/file from monthly summary to period summary.
- Added shared preset labels for `today`, `7d`, `30d`, and `90d`; period summary and daily totals now render labels such as `今日平均响应`, `7 天请求`, and `30 天费用`.
- Replaced the backend admin seed `dashboard.stats.monthly` strings with `dashboard.stats.period` templates and removed the unused daily total label keys.
- Refinement validation passed: `pnpm lint:frontend`, `pnpm build:frontend`, `jq empty apps/hook_backend/src/migration/defaults/i18n/admin.cn.json apps/hook_backend/src/migration/defaults/i18n/admin.en.json`, and `git diff --check`.
- Applied the same period-label pattern to the model cost chart title: it now renders `今日模型成本`, `7 天模型成本`, `30 天模型成本`, or `90 天模型成本`.
- Model cost title validation passed after fixing an initial prop placement mistake caught by `pnpm build:frontend`: `pnpm lint:frontend`, `pnpm build:frontend`, `jq empty ...admin.cn.json ...admin.en.json`, and `git diff --check`.
- Added backend API pagination for the daily stats table. `DashboardOverviewRequest` now accepts `page` and `page_size`, and `DashboardDailyStats.day_page` returns the existing page shape `{ items, total, page, page_size }`.
- The daily table now uses `day_page.items` and `day_page.total`; chart/totals data still use full-period `daily.days` so graph and aggregate semantics are unchanged.
- Pagination validation passed: `timeout 60s cargo test -p dashboard`, `timeout 60s cargo test -p storage dashboard`, `pnpm lint:frontend`, `pnpm build:frontend`, and `git diff --check`. The first `cargo test -p dashboard` attempt timed out while waiting for cargo locks, then passed on rerun.
