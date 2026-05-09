# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Make an invalid or rejected refresh token end the frontend session cleanly instead of surfacing a React console error.
- Return proper HTTP unauthorized status for backend user authentication failures.
- Keep invalid sign-in credentials user-facing message unchanged.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change the global API envelope shape.
- Do not add silent fallback or mock authentication paths.
- Do not touch unrelated wallet work currently present in the worktree.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Frontend uses Next.js and axios under `apps/hook_frontend`.
- Backend user API lives in `crates/user`; application auth errors must remain typed.
- Backend tests must use the repository's 60-second timeout policy.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024, TypeScript/Next.js
- **Package manager**: pnpm
- **Test framework**: Rust `cargo test`; no frontend JS test runner configured
- **Build command**: `pnpm build:frontend`, `just test`
- **Existing test count**: not precomputed

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — no external service required for unit/API tests.
- [x] Breaking changes to existing code — user auth API error status expectations are affected.
- [x] Large file generation — no large files expected.
- [x] Long-running tests — backend tests will use `timeout 60`.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Updated backend user API error status mapping and messages.
- Updated frontend JWT refresh-session handling.
- Validation commands and outcome recorded in PROGRESS.md.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Invalid refresh token does not throw through `AuthProvider`.
- [ ] Backend unauthorized user API errors return HTTP 401.
- [ ] Invalid sign-in credentials still return `"username or password is incorrect"`.
- [ ] Relevant Rust and frontend checks pass or any blocker is documented.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
timeout 60 cargo test -p user && pnpm lint:frontend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
