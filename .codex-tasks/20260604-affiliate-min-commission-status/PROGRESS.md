# Progress

## 2026-06-04

- Inspected recharge settlement, affiliate commission schema, system settings, admin/user affiliate queries, CSV exports, and frontend tables.
- Current settlement always credits wallet before inserting affiliate commission records. Minimum-threshold behavior needs a failed record path without wallet creation or wallet transaction.
- Current aggregate queries sum all commission rows. After adding failed rows, paid commission totals should only sum successful rows while detail lists continue to show every row.
- Added affiliate minimum commission setting, commission status, and failure reason. Settlement now records commissions below the minimum as failed with reason `below_min_commission_amount`, without creating a referrer wallet or wallet transaction.
- Updated admin and user affiliate record tables to use horizontal scrolling, non-wrapping action buttons/cells, and status/failure reason columns.
- Updated aggregate queries so paid commission totals only include `status = 'success'`, while detail lists and CSV exports include both successful and failed records.
- Validation passed: `cargo check -p backend -p user -p storage -p types -p recharge -p setting`; `cargo fmt --all --check`; `timeout 60 cargo test -p setting affiliate -- --nocapture`; `timeout 60 cargo test -p storage affiliate_commission -- --nocapture`; `pnpm lint:frontend`; `pnpm build:frontend`; `git diff --check`.
