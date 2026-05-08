# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Persist JWT access and refresh tokens in `localStorage` instead of `sessionStorage`.
- Persist language selection only in `localStorage`.
- Remove cookie-based language detection and cookie writes from the frontend i18n flow.
- Remove the language-specific `CONFIG.isStaticExport` split where it only exists to choose cookie vs localStorage.

## Non-Goals

- Do not change backend authentication semantics.
- Do not introduce cookie authentication.
- Do not refactor unrelated static export data-loading branches.
- Do not alter settings/theme cookies outside the language flow.

## Constraints

- Follow existing TypeScript and Next.js patterns.
- Keep failures explicit; do not add fallback persistence layers.
- Keep changes scoped to authentication persistence and language persistence.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js
- **Package manager**: `pnpm`
- **Validation**: frontend lint/build where feasible

## Risk Assessment

- [x] `localStorage` is browser-only, so server layout cannot read selected language without cookies.
- [x] Initial server-rendered `<html lang>` must use default language when cookies are removed.
- [x] Existing dirty worktree has unrelated changes; this task must not revert them.

## Deliverables

- Updated auth session storage implementation.
- Updated i18n provider and language persistence implementation.
- Removed language cookie reads/writes from the active code path.

## Done-When

- [ ] No `sessionStorage` references remain in frontend source.
- [ ] No language cookie read/write references remain in frontend locale flow.
- [ ] JWT token keys are read, written, and removed through `localStorage`.
- [ ] i18n client detection uses `localStorage`.
- [ ] Frontend lint/build validation has been run or the exact blocker is recorded.

## Final Validation Command

```bash
pnpm lint:frontend
```
