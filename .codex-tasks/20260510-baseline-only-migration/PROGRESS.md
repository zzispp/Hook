# Progress

## Recovery

- 2026-05-10: Task initialized; inspecting current migration chain before consolidation.
- 2026-05-10: Baseline now owns the final schema for users, RBAC, wallets, models, billing groups, API tokens, billing-group model bindings, and system settings.
- 2026-05-10: Removed all `m20260510_*` migration modules from the registry and filesystem; only `m20260508_000001_create_baseline` remains registered.
- 2026-05-10: Validation passed with `cargo fmt --all`, `cargo check --workspace`, and `cargo test -p backend`.
- 2026-05-10: `cargo run -p backend -- migration status` fails on the existing local database because `seaql_migrations` still records the removed development migrations. This is expected for a baseline reset and needs a clean/rebuilt development database.
