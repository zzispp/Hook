# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Fix streamed proxy requests that remain `pending` in request records after the upstream stream completes.
- Investigate and fix frequent streamed proxy failures reported as `stream disconnected before completion: Transport error: network error: error decoding response body`.
- Compare relevant behavior with `/Users/bubu/ZwjProjects/Aether` and `/Users/bubu/ZwjProjects/new-api` before changing Hook.

## Non-Goals

- Do not add silent fallback, mock success, or compatibility-only branches.
- Do not change unrelated admin UI behavior.
- Do not alter provider business thresholds unless the read sites and semantics are verified first.

## Constraints

- Follow backend AGENTS.md rules under `apps/hook_backend`.
- Use TDD for backend changes where feasible.
- Keep failures visible and explicit.
- Use repository validation commands with 60-second test wrapper where applicable.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024 / Axum backend, TypeScript frontend
- **Package manager**: cargo, pnpm
- **Test framework**: Rust unit/integration tests
- **Build command**: `cargo check -p hook_backend`
- **Existing test count**: not yet measured

## Risk Assessment

- [x] External dependencies (APIs, services) — local backend is running at `127.0.0.1:5555`; upstream availability still to be observed.
- [ ] Breaking changes to existing code — pending call-site review.
- [x] Large file generation — not expected.
- [x] Long-running tests — backend policy requires `just test` timeout wrapper.

## Deliverables

- Focused backend fix and tests for stream completion request record behavior.
- Root-cause evidence for stream disconnect behavior and a code fix when confirmed in Hook.
- Validation command output summary.

## Done-When

- [ ] A failing test captures the current stream completion/record bug or an equivalent unit boundary.
- [ ] Production code updates records on successful stream completion.
- [ ] Stream transport no longer treats valid streaming termination as an incomplete network decode failure.
- [ ] Relevant Rust checks pass or any blocker is explicitly reported.

## Final Validation Command

```bash
just test
```
