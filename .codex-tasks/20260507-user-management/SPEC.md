# User Management Backend

## Goal

Implement an Axum backend user management feature in a multi-crate Rust workspace.

## Requirements

- Use Axum for HTTP routing.
- Use Toasty ORM.
- Use PostgreSQL at `localhost:5433` with database/user `postgres`.
- Provide sign in, sign up, create user, delete user, update user, and paginated user list APIs.
- User fields: username, password, email, role, status.
- Crates are split by module, not by layer.
- Use a dedicated constants crate.
- Use a dedicated config crate backed by YAML through `config-rs`.
- Do not prefix crate package names with `hook_`.
- The user module crate contains its own domain, application, infra, and API layers.
- Keep ORM-specific code out of business logic modules.

## Boundaries

- No mock success paths or silent fallback behavior.
- Fail startup clearly when database connection/schema setup fails.
- Passwords must not be returned in API responses.
