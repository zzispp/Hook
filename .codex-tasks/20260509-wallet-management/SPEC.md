# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Implement the Hook frontend wallet management/center page at `/dashboard/wallet`.
- Use the existing wallet API baseline: balance summary and transaction list.
- Add a tab scaffold with `资金流水` as the first tab for future wallet tabs.
- Add a filter area inspired by aether: search, category, reason, amount direction, balance type, and reset.
- Let every transaction open a details dialog containing audit fields like aether's 流水详情.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- No backend API changes.
- No admin wallet management, recharge, refund, redeem code, or payment workflow.
- No compatibility layer for historical frontend shapes.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Next.js app router, React 19, MUI 7, TypeScript.
- Use existing `src/lib/axios.ts` endpoints and API response envelope.
- Keep implementation scoped to wallet files.
- Client-side filtering only for the current loaded page because the backend endpoint exposes only pagination.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: `TypeScript / Next.js`
- **Package manager**: `pnpm`
- **Test framework**: `No JS unit test runner configured`
- **Build command**: `pnpm --filter hook_frontend build`
- **Existing test count**: `N/A for frontend`

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — existing endpoints confirmed in `src/lib/axios.ts` and Rust wallet routes.
- [x] Breaking changes to existing code — impact limited to the missing wallet page implementation.
- [x] Large file generation — not applicable.
- [x] Long-running tests — frontend build/lint only.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- `apps/hook_frontend/src/sections/wallet/*`
- Updated wallet page import target remains valid.
- Validation output from lint/build.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] `/dashboard/wallet` compiles and renders balance summary, tabs, filters, table, and transaction detail dialog.
- [ ] TypeScript and ESLint checks pass for the frontend.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm --filter hook_frontend lint && pnpm --filter hook_frontend build
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Open `/dashboard/wallet`.
2. Confirm `资金流水` tab is visible.
3. Apply filters and reset them.
4. Click a transaction row and confirm the audit dialog displays balance changes and identifiers.
