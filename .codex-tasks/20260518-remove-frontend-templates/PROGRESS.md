# Progress

## 2026-05-18

- Started implementation from the approved removal plan.

- Removed template-only actions/types/mock modules and pruned template-only UI component branches/dependencies.
- Fixed hydration mismatch by initializing client i18n from the server language for first render and making relative-time formatting locale-explicit.
- Validation passed: `pnpm --filter hook_frontend lint` and `pnpm --filter hook_frontend build`.
