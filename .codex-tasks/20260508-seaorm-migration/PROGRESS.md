# Progress Log

## Session Start

- **Date**: 2026-05-08
- **Task name**: `20260508-seaorm-migration`
- **Scope**: replace Toasty ORM/schema/default init with SeaORM migrations.

## Milestone 1: Initial inspection

- **Status**: DONE
- **Root causes**:
  - Current backend uses Toasty model macros and `toasty::Db` repositories across users, RBAC, and models.
  - Current schema path has both `schema bootstrap/push` and Toasty migration commands.
  - Current default RBAC/menu/user seed data is runtime init code under `apps/hook_backend/src/init`.
- **Migration target**:
  - Use SeaORM for runtime storage.
  - Use SeaORM Migrator for schema and seed data, so ordered migrations become the single source of database initialization and evolution.

## Milestone 2: Migration design

- **Status**: DONE
- **Migration design**:
  - Runtime storage uses SeaORM `DatabaseConnection` and SeaORM entities in `crates/storage`.
  - Schema creation and default RBAC/menu/API seed data live in the baseline SeaORM migration.
  - Startup no longer pushes schema or runs default init. It only rebuilds the RBAC cache after connecting.
  - SeaORM's `seaql_migrations` table is the migration idempotency source.
  - The baseline keeps the existing table names and fields for users, RBAC, menus, and model management.

## Milestone 3: Implementation

- **Status**: DONE
- Replaced Toasty storage repositories with SeaORM queries.
- Replaced schema/init commands with `migration up/down/status/fresh/refresh/reset`.
- Removed runtime init/default seed modules and Toasty schema push code.
- Updated README and justfile for the SeaORM migration workflow.

## Milestone 4: Validation

- **Status**: DONE
- `cargo fmt --all -- --check`: passed.
- `cargo check`: passed.
- `cargo clippy -p backend --all-targets -- -D warnings`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `just test`: could not run because `just` is not installed in the current shell.
- Equivalent 60-second test wrapper from `justfile` with `cargo test`: passed.
- `cargo run -p backend -- migration status`: command executed successfully.
- `cargo run -p backend -- migration up`: applied successfully; a second `up` also succeeded.
