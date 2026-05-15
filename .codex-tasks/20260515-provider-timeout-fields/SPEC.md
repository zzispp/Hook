# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Add provider timeout inputs to the admin create/edit provider modal.
- Make stream first-byte timeout default to 30 seconds.
- Keep non-stream request timeout default at 300 seconds.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change runtime timeout semantics beyond the requested defaults and exposed fields.
- Do not add fallback behavior, compatibility shims, or unrelated provider settings.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Follow the repository i18n rule: admin UI copy is backend-controlled through seed files.
- Preserve existing provider payload/update flow.
- Backend provider defaults are only applied when creating a provider without explicit timeout values.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: `TypeScript Next.js frontend + Rust backend`
- **Package manager**: `pnpm`
- **Test framework**: `lint/build checks; Rust just test wrapper when backend behavior changes`
- **Build command**: `pnpm lint:frontend`
- **Existing test count**: `not measured for this scoped UI/config change`

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — not required.
- [x] Breaking changes to existing code — scoped to provider admin fields and create default.
- [x] Large file generation — not applicable.
- [x] Long-running tests — frontend lint is the target validation.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Updated provider admin modal with timeout number inputs.
- Updated provider form state and payload mapping.
- Updated backend default stream first-byte timeout to 30 seconds.
- Updated backend i18n seed copy for new labels.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Create and edit provider modal can submit `request_timeout_seconds` and `stream_first_byte_timeout_seconds`.
- [ ] Existing provider values populate the edit modal.
- [ ] Blank fields submit documented defaults: 300s non-stream, 30s stream first byte.
- [ ] Frontend lint passes or any failure is explicitly reported.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm lint:frontend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
