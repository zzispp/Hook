# Progress

## 2026-05-10

- Started system settings implementation.
- Reviewed current integration points: user sign-up/admin create user are in `crates/user`, wallet gift balance and ledger are in `crates/wallet`/`crates/storage/src/wallet`, token rate config currently stores `rate_limit_rpm=0` for follow-system, and admin menu/API defaults are migration-seeded.
- Added `setting` crate, `SystemSettings` DTOs, `SettingStore`, admin settings API, startup wiring, and migration `m20260510_000007_create_system_settings` with RBAC menu/API bindings.
- Wired registration settings into user sign-up: `allow_registration=false` rejects public sign-up; successful sign-up grants `default_user_grant` into gift balance and writes `initial_user_grant` wallet ledger.
- Wired token settings into token listing: expired token cleanup runs when `auto_delete_expired_tokens=true`, and tokens with `rate_limit_rpm=0` display the system `default_rate_limit_rpm`.
- Added admin settings page at `/dashboard/admin/settings` with Site, Base, and Token sections, plus a rate-limit column in user/admin token tables.
- Validation passed: `cargo fmt --all`, `cargo check --workspace`, `pnpm lint:frontend`, `cargo run -p backend -- migration up`, and `perl -e 'alarm 60; exec @ARGV' cargo test --workspace`.
