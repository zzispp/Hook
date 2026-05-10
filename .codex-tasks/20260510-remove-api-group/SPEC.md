# Remove API Permission Group Field

## Goal

Remove the hand-written API permission group field from frontend and backend, including the database column via an additive migration. Update API menu selection to render menu hierarchy with indentation based on bound menu parent relationships.

## Scope

- Remove `api_permissions.group` from active Rust types, storage entity, mappers, and API payloads.
- Add a new SeaORM migration that drops the `group` column from `api_permissions`.
- Remove API group display/input from the admin API management UI.
- Render API menu options in tree order with indentation in API menu selection.
- Keep existing migrations immutable except for active default seed helpers if required by current compile-time definitions.

## Validation

- `cargo fmt --all`
- `cargo check --workspace`
- `pnpm lint:frontend`
- `cargo run -p backend -- migration up`
- `perl -e 'alarm 60; exec @ARGV' cargo test --workspace`
