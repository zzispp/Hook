# Progress

- 2026-06-01T02:00:00+08:00: Confirmed config admin is represented by `User.system = true`; account mutation endpoints are service-layer methods in `crates/user/src/application/service/use_cases.rs`; profile UI renders password/provider cards unconditionally.
- 2026-06-01T02:07:00+08:00: Added service-layer system user rejection for password email code, password change, identity unlink, OAuth profile email, and wallet email binding paths. Profile UI now gates mutation cards on `!user.system`.
- 2026-06-01T02:18:00+08:00: Split large helpers into `use_cases/helpers.rs` and test support helpers. Validation passed: `git diff --check`, `pnpm lint:frontend`, and `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p user --lib --bins`.
