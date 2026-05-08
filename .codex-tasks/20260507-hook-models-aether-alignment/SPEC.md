# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Understand Hook frontend and backend architecture, code style, and directory constraints.
- Study Aether's model management module, especially `/admin/models`, backend model services, schema, and `model.dev` fetching.
- Add equivalent model management functionality to Hook frontend and backend with schema/field alignment as close to 1:1 as Hook architecture permits.

## Non-Goals

- Do not redesign unrelated Hook auth, RBAC, or layout systems.
- Do not add mock success paths, silent fallbacks, or compatibility shims.
- Do not implement unrelated Aether modules outside model functionality unless required by the model feature boundary.

## Constraints

- Follow Hook root `AGENTS.md` and backend `AGENTS.md`.
- Use TDD for backend behavior where feasible.
- Keep Rust business rules in shared crates and composition wiring in `apps/hook_backend`.
- Keep TypeScript formatting and imports consistent with the existing Next.js frontend.
- Backend tests should run through repository timeout wrappers where practical.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024, TypeScript/Next.js
- **Package manager**: pnpm
- **Test framework**: Rust unit tests, Next.js build/lint for frontend
- **Build command**: `just check`, `pnpm build:frontend`
- **Existing test count**: To be discovered

## Risk Assessment

- [ ] External dependencies (model.dev API) — contract and authentication requirements discovered.
- [ ] Breaking changes to existing code — routing and schema impact assessed.
- [ ] Large file generation — not expected.
- [ ] Long-running tests — use `just test` wrapper for Rust.

## Deliverables

- Aether model feature map and Hook implementation plan captured in progress notes.
- Hook backend schema, storage, types, service, and API routes for model management.
- Hook frontend admin model management page and API client/types.
- Automated validation for backend/frontend changes where feasible.

## Done-When

- [ ] Hook exposes model management APIs aligned with Aether's model module.
- [ ] Hook can fetch and store model.dev models.
- [ ] Frontend admin model page supports the aligned workflow.
- [ ] Verification commands pass or any failures are explicit and diagnosed.

## Final Validation Command

```bash
just check && pnpm lint:frontend && pnpm build:frontend
```

## Demo Flow

1. Start Hook backend and frontend.
2. Open the admin model page.
3. Trigger model.dev fetch.
4. Verify fetched models appear with aligned fields.
