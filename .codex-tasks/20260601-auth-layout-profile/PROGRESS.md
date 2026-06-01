# Progress

## 2026-06-01

- Read the Minimal auth split layout and found its `AuthSplitContent` source is effectively the same as the original project file.
- Current first auth edit changed content alignment but did not solve the user's observed layout, so runtime box measurement is needed before the next layout change.
- Started tracing `/dashboard/profile/` and found the visible 403 text comes from `DashboardRouteGuard`, which checks whether the current route exists in navbar data.
- Fixed auth split centering by giving the auth `main` a viewport-based minimum height and centering the content column across breakpoints.
- Browser measurements at 1440x900 showed both sign-in and sign-up content columns centered at viewport center Y=450.
- Corrected the profile 403 fix to use seeded RBAC data instead of a frontend guard exception.
- Added `dashboard_profile` to the default menu definitions and default user/admin role menu codes; `/api/account/profile` remains covered by authenticated base API rules.
- Added admin i18n nav labels for `dashboard_profile`.
- Validation passed: `git diff --check`, `pnpm lint:frontend`, `cargo check --workspace`, and `cargo test -p backend migration::defaults`.
