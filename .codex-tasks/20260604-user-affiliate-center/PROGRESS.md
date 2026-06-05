# Progress

## 2026-06-04

- Created Epic tracking for user affiliate center implementation.

- Implemented user affiliate types, storage queries, application use case, API state wiring, and repository forwarding. `cargo check -p types -p storage -p user` passed.

- Added user-facing affiliate menu/API permissions, i18n seed keys, `/dashboard/affiliate` page, referrals and commissions tables, CSV export, and removed wallet affiliate card/data loading.
- Validation passed: cargo fmt --all; cargo check -p backend -p user -p storage -p types; cargo test -p backend migration::defaults -- --nocapture; cargo test -p user api::routes::tests -- --nocapture; pnpm lint:frontend; pnpm build:frontend.
