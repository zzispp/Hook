# Progress

## 2026-05-14

- Confirmed backend pagination exists for announcements and tickets while frontend fixed both to first 50 rows.
- Confirmed notification resources currently disable focus revalidation and have no polling.
- Added backend ticket attention ordering: unread counterparty activity first, unfinished statuses next, then newest message.
- Added pagination controls to admin announcement table and ticket list panel using backend page/page_size.
- Added notification freshness policy: focus/reconnect revalidation, visible-window 30s polling, drawer-open refresh, and tab-switch refresh.
- Validation passed: `cargo fmt --all`, `cargo check --workspace`, `pnpm lint:frontend`, `pnpm build:frontend`, and 60-second-wrapped `cargo test --workspace`.
