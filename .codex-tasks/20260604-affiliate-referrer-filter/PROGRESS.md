# Affiliate Referrer Filter Progress

## 2026-06-04

- Confirmed the admin affiliate rebind selector now receives the relation user id and filters candidates to regular users only.
- Confirmed affiliate-code and user search modes share the same candidate list, so the current user and admin-role users are not shown as selectable referrers.
- Confirmed storage-level relation updates reject non-regular referrers, so manually submitted admin affiliate codes cannot bypass the UI.

## Validation

- `cargo fmt --all --check`
- `pnpm lint:frontend`
- `cargo check -p backend -p user -p storage -p types`
- `pnpm build:frontend`
- `cargo test -p user admin_affiliate_rebind_rejects_admin_referrer -- --nocapture`
