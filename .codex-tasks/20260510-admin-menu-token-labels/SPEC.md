# Admin Menu And Token Labels

## Goal

Fix default RBAC menu bindings so administrators do not receive user-facing wallet, token, or model catalog menus, and make user/admin token menu labels distinct.

## Scope

- Update baseline defaults for admin role menu bindings.
- Keep user role menu bindings on dashboard, model catalog, wallet center, and user API tokens.
- Rename user-facing token labels so they no longer collide with admin token management labels.
- Validate with targeted Rust checks and frontend type checking.
