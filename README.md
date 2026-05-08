# Hook

Rust and pnpm monorepo for the Hook backend, frontend, mock API, and shared crates.

## Backend Logging

Backend logging uses the internal `hook_tracing` crate. Runtime log verbosity is
configured in `config/config.yaml`:

```yaml
tracing:
  log_level: "info"
```

Valid levels are `off`, `error`, `warn`, `info`, `debug`, and `trace`. Invalid
values fail startup explicitly.

## Backend Database

The backend uses SeaORM for runtime database access and SeaORM Migrator for schema
creation, schema evolution, and default seed data.

Startup does not create or mutate tables. Run migrations explicitly before serving
against a fresh or upgraded database.

### Run Migrations

Apply all pending migrations:

```bash
cargo run -p backend -- migration up
```

or:

```bash
just backend-migration "up"
```

Check migration state:

```bash
cargo run -p backend -- migration status
```

Roll back migrations:

```bash
cargo run -p backend -- migration down
cargo run -p backend -- migration down 2
```

`down` without a step count rolls back one migration.

### Reset Commands

SeaORM Migrator also exposes database reset helpers:

```bash
cargo run -p backend -- migration fresh
cargo run -p backend -- migration refresh
cargo run -p backend -- migration reset
```

- `fresh`: drop all migration-managed tables, then run every migration again.
- `refresh`: roll back all applied migrations, then run every migration again.
- `reset`: roll back all applied migrations.

### Custom Config

Pass a config file before the migration command:

```bash
cargo run -p backend -- --config config/config.yaml migration up
just backend-migration-config config/config.yaml "up"
```

### Baseline Defaults

The first migration creates the baseline tables and seeds built-in RBAC data,
navigation menus, API permissions, and model tables. SeaORM tracks applied
migrations in `seaql_migrations`, so `migration up` only applies pending
migrations.
