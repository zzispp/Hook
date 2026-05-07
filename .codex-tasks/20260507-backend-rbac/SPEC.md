# Backend RBAC Implementation

## Goal

Implement backend support for a config-backed immutable administrator, role/API/menu RBAC management, backend-rendered menu data, auth middleware, API whitelist, and Redis-backed permission/menu cache.

## Scope

- Rust backend only.
- Preserve existing crate boundaries: config, types, storage, user, backend composition root.
- Add a dedicated RBAC crate for business logic and API routes.
- Keep failures explicit; no DB fallback when Redis cache is unavailable or missing.
- Do not modify frontend behavior in this task.

## Acceptance Criteria

- Configuration supports admin, auth whitelist, and Redis settings.
- The configured admin appears in user listing and cannot be disabled, replaced, or deleted.
- Protected APIs require bearer auth unless matched by config whitelist.
- RBAC API permissions and menu permissions are stored in DB and cached in Redis.
- API/menu/role mutations rebuild Redis cache.
- `/api/navbar` returns the authenticated user's backend-filtered menu tree.
- Backend tests cover admin immutability, middleware whitelist/auth flow, RBAC cache refresh, and menu filtering.
- Rust backend validation runs with the repository's timeout policy when feasible.
