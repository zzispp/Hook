# Menu API RBAC Refactor

## Goal

Refactor RBAC so roles grant menus only, menus bind APIs, and runtime API authorization is derived from role-menu plus menu-api bindings.

## Scope

- Add backend schema/storage/service support for menu API bindings.
- Remove role direct API permission management from the target API and frontend UI.
- Seed and migrate default menu/API bindings with CNY wallet and existing dashboard permissions preserved.
- Add a menu management UI for binding APIs to menu items.
- Keep authorization failures explicit; no silent fallback or compatibility behavior.

## Non-goals

- Reworking wallet business logic beyond permission alignment.
- Adding new role concepts beyond current menu-driven model.
- Introducing mock authorization paths.

## Validation

- Rust formatting and cargo checks/tests where feasible.
- Frontend lint/build where feasible.
- Manual code-path validation for migration and route/API authorization chain.
