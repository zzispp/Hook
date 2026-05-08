# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Replace the current Toasty ORM path with SeaORM for backend storage.
- Move schema creation and default seed data into ordered, idempotent migrations.
- Remove startup-time default initialization code once migrations own defaults.
- Update README and command docs to describe the new migration workflow.

## Assumptions

- User's `sem-orm` means `SeaORM` / `sea-orm`.

## Non-Goals

- Do not change API behavior, RBAC semantics, auth semantics, or frontend contracts.
- Do not add silent fallback schema creation.
- Do not preserve Toasty compatibility paths after the replacement is complete.

## Done-When

- Backend storage compiles and runs against SeaORM.
- Migrations create tables and default data in ordered up/down files.
- Startup no longer performs init/default seed logic.
- README documents migration commands and no longer documents Toasty schema/bootstrap.
- Rust checks pass or failures are explicitly documented.
