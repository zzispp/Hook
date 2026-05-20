# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape

- **Shape**: `single-full`

## Goals

- Fix the admin provider endpoint body rule editor so entering an override for `instructions` does not crash the console during rendering.

## Non-Goals

- Do not change backend body rule semantics.
- Do not add compatibility fallbacks or silent parsing suppression in save-time validation.

## Constraints

- Frontend-only fix.
- Keep invalid intermediate JSON visible through validation, but do not crash while the user is editing.

## Environment

- **Project root**: `<auto>`
- **Language/runtime**: `<auto>`
- **Package manager**: `<auto>`
- **Test framework**: `<auto>`
- **Build command**: `<auto>`
- **Existing test count**: `<auto>`

## Risk Assessment

- [ ] External dependencies (APIs, services) - availability confirmed?
- [ ] Breaking changes to existing code - impact assessed?
- [ ] Large file generation - disk space sufficient?
- [ ] Long-running tests - timeout configured?

## Deliverables

- Updated provider endpoint body rule editor logic.
- Validation run for the frontend package.

## Done-When

- [ ] Typing an `instructions` body rule override no longer throws `Unexpected end of JSON input` in the browser console.
- [ ] Frontend lint passes.

## Final Validation Command

```bash
pnpm --filter hook_frontend lint
```

## Demo Flow (optional)

1. Open admin endpoint management.
2. Add or edit a body rule with path `instructions`.
3. Type an override value and confirm the console stays clean.
