# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Allow admin provider model tests to run even when the provider itself is disabled.
- Keep the existing validation for test-selected endpoint, key, and model bindings unchanged unless evidence shows they are also wrong.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change provider runtime routing behavior for normal traffic.
- Do not broaden model test eligibility for inactive keys, endpoints, or model bindings in this task.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Follow existing Rust model-test selection flow in `apps/hook_backend/src/llm_proxy/model_test/`.
- Keep failures explicit; do not add fallback behavior.
- Respect the repository's task and validation conventions.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace, Next.js frontend
- **Package manager**: pnpm, cargo, just
- **Test framework**: Rust `#[test]`
- **Build command**: `just build`
- **Existing test count**: not enumerated; using targeted Rust tests for validation

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — availability confirmed?
- [x] Breaking changes to existing code — impact assessed?
- [ ] Large file generation — disk space sufficient?
- [x] Long-running tests — timeout configured?

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Backend model-test selection update that stops rejecting inactive providers.
- Regression test covering provider test behavior for inactive providers.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] `POST /api/admin/providers/{provider_id}/models/{model_id}/test` no longer rejects a disabled provider only because `provider.is_active` is false.
- [ ] Existing targeted model-test selection tests still pass, plus a new inactive-provider regression test.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
cargo test -p backend fixed_parts_
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. In admin provider details, disable a provider that still has a valid testable endpoint and key.
2. Open the model test dialog for a bound model and submit a test request.
3. Confirm the request proceeds past provider lookup instead of failing with `provider not found or inactive`.
