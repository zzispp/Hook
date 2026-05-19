# Progress

## 2026-05-19

- Created task tracking files.
- Compared aether. Its analytics query treats `active` as a semantic filter mapping to `pending + streaming`, while preserving raw status filters. Hook should follow this pattern instead of redefining `streaming`.
- Implemented Hook changes:
  - backend request-record list maps `status=active` to `pending + streaming`;
  - frontend status dropdown adds a semantic active/in-progress option using existing `requestRecords.inProgress` copy;
  - active polling now locally removes records that no longer match the current status filter before refreshing the list.
- Validation:
  - `pnpm --filter hook_frontend exec eslint src/sections/admin/request-records-view.tsx src/sections/admin/request-records-utils.ts src/sections/admin/request-records-polling.ts` passed.
  - `git diff --check` on touched source files passed.
  - Full `pnpm --filter hook_frontend lint` is blocked by an existing import order error in `apps/hook_frontend/src/auth/view/jwt/jwt-sign-up-view.tsx`.
  - `pnpm --filter hook_frontend exec tsc --noEmit` is blocked by a malformed generated `.next/dev/types/routes.d.ts`.
  - `cargo test -p provider validate_request_record_list` and `cargo check -p provider` are blocked by an existing compile error in `crates/storage/src/user/repository.rs` (`Condition` is not imported).
