# Auth Layout And Profile Access

## Objective

Fix two user-visible frontend issues:

- Login and registration form content should be vertically centered, matching the Minimal reference behavior.
- `/dashboard/profile/` should be accessible to normal authenticated users instead of showing the dashboard permission denied page.

## Constraints

- Keep changes scoped to existing auth/dashboard layout and RBAC navigation patterns.
- Do not add silent fallbacks or fake permission bypasses.
- Validate with lint and source/runtime evidence.
