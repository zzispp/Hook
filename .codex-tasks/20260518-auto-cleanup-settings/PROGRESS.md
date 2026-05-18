# Progress

## 2026-05-18
- Started auto-cleanup settings implementation.
- Added scheduled auto-cleanup system settings for request records and performance monitoring snapshots.
- Wired the settings through backend types, storage mapping, validation, baseline seed, frontend form state, and backend i18n seed resources.
- Updated request record cleanup and performance monitoring cleanup workers to read the current cleanup settings each loop.
- Validation completed: `cargo fmt`, `just check`, `just test`, `pnpm lint:frontend`, `pnpm build:frontend`.
