# React Bits TSX Migration

## Goal

Rename all currently changed React Bits JavaScript files to TypeScript:

- `.jsx` -> `.tsx`
- `.js` -> `.ts`

## Scope

Only existing current-worktree changed source files under `apps/hook_frontend/src/react-bits` are in scope. CSS, assets, deleted legacy files, and unrelated existing TypeScript files are out of scope.

## Acceptance Criteria

- No changed React Bits source files remain with `.js` or `.jsx` extensions.
- React components compile under the frontend TypeScript configuration.
- Existing landing-page behavior and theme changes remain intact.
- `pnpm --filter hook_frontend lint` passes.
- `pnpm --filter hook_frontend build` passes.
