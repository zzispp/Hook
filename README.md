# Hook

Rust and pnpm monorepo for the Hook backend, frontend, mock API, and shared crates.

## Backend Database Schema

The backend uses Toasty for ORM schema metadata. There are three schema-related paths, and they are intentionally different.

### Normal Startup

`config/config.yaml` contains:

```yaml
database:
  push_schema_on_startup: false
```

Keep this `false` for normal development and deployment. Backend startup should not silently create or mutate database tables.

### Bootstrap An Empty Database

Use bootstrap when preparing a local or fresh database:

```bash
cargo run -p backend -- schema bootstrap
```

or:

```bash
just bootstrap-backend-schema
```

`schema bootstrap` is intentionally three-state:

- If none of the backend-managed tables exist, it creates the full Toasty schema.
- If all backend-managed tables already exist, it exits successfully.
- If only some backend-managed tables exist, it fails and lists present and missing tables.

This avoids fake idempotency. A partial database is not treated as healthy because existing tables may still have the wrong columns, indexes, or constraints.

### Raw Schema Push

The lower-level command is:

```bash
cargo run -p backend -- schema push
```

This calls Toasty's `Db::push_schema()` directly. It is not idempotent and is not a migration system. If a managed table already exists, PostgreSQL can return an error such as `relation "users" already exists`.

Prefer `schema bootstrap` for empty database setup.

### Toasty Migrations

For schema evolution after the initial baseline, use Toasty's migration runner:

```bash
cargo run -p backend -- migration generate
cargo run -p backend -- migration apply
```

or:

```bash
just backend-migration "generate"
just backend-migration "apply"
```

`migration apply` is migration-level idempotent: it records applied migrations in `__toasty_migrations` and only applies pending migrations from Toasty's history files.

`migration snapshot` can print the current Toasty schema:

```bash
cargo run -p backend -- migration snapshot
```

Do not mix direct `schema push` with migration history for long-lived databases. For an existing database that was created before Toasty migration files existed, create a baseline first, then use `migration generate/apply` for later changes.
