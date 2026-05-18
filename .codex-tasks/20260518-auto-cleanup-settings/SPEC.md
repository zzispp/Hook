# System Settings Auto Cleanup

## Goal
Add a real scheduled auto-cleanup section to system settings so retained operational data can be cleaned periodically instead of growing without bound.

## Scope
- Inspect existing system settings persistence and cleanup jobs.
- Add backend setting fields and cleanup scheduling logic where needed.
- Add a system settings section that edits the cleanup settings.
- Update backend i18n seed entries for new admin copy.

## Validation
- `pnpm lint:frontend`
- `pnpm build:frontend`
- `just check`
- `just test`
