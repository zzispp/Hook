# Progress Log

## Session Start

- **Date**: 2026-05-08
- **Task name**: `20260508-user-model-dashboard`
- **Scope**: normal-user auth/RBAC fix plus user-visible model catalog dashboard.

## Root Cause

- `POST /api/auth/sign-in` succeeds for the normal account in `/Users/bubu/Downloads/localhost.har`.
- The next session check calls `GET /api/auth/me` and receives business error `forbidden`.
- Backend auth middleware authenticates first, then authorizes the route through RBAC.
- `ensure_default_rbac` currently binds all default APIs and menus only to the configured admin role.
- The default normal `user` role exists but has no `/api/auth/me`, `/api/navbar`, or model catalog permissions.
- The fix is to bind a minimal user role permission set instead of granting admin APIs or bypassing auth.
