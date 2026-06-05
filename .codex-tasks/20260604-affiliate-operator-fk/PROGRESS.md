# Affiliate Operator FK Fix Progress

## 2026-06-04

- Confirmed `affiliate_relation_changes.operator_user_id` is nullable but has a foreign key to `users.id`.
- Confirmed the failing request used the virtual system admin user from auth, which is not stored in the `users` table.
- Changed admin affiliate relation updates to pass `None` for virtual system operators and preserve `Some(user_id)` for database-backed operators.
- Added focused tests for both API operator mapping and admin affiliate relation audit records.

## Validation

- `cargo fmt --all --check`
- `cargo check -p backend -p user -p storage -p types`
- `timeout 60 cargo test -p user admin_affiliate -- --nocapture`
