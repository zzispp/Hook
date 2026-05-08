# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Remove the account drawer promotional upgrade block matching the provided DOM.
- Delete the static rocket image used only by that block.
- Keep unrelated navigation/user account UI intact.

## Non-Goals

- Do not redesign the account drawer.
- Do not remove dashboard `NavUpgrade` user-account content unless it is required for the requested block removal.
- Do not touch unrelated existing worktree changes.

## Constraints

- Follow repository TypeScript and React patterns.
- Keep failures visible; do not add fallback UI or compatibility paths.
- Scope edits to the promotional block and its exclusive asset.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / React / Next.js
- **Package manager**: `pnpm`
- **Test framework**: no JavaScript test runner configured
- **Build command**: `pnpm build:frontend`
- **Existing test count**: not applicable for frontend

## Risk Assessment

- [x] External dependencies (APIs, services) — no runtime dependency needed for this static removal.
- [x] Breaking changes to existing code — import/render references will be searched and removed.
- [x] Large file generation — not applicable.
- [x] Long-running tests — lint/build are bounded by command timeout.

## Deliverables

- Remove `UpgradeBlock` rendering from the account drawer.
- Remove the now-unused `UpgradeBlock` component.
- Delete `apps/hook_frontend/public/assets/illustrations/illustration-rocket-small.webp`.

## Done-When

- [ ] No references remain to `UpgradeBlock` or `illustration-rocket-small.webp`.
- [ ] Frontend lint passes or exposes only unrelated pre-existing issues.

## Final Validation Command

```bash
pnpm lint:frontend
```
