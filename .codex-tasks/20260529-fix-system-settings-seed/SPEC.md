# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Fix the baseline `system_settings` seed so PostgreSQL receives boolean values for boolean auth provider columns.
- Add focused test coverage that proves seeded columns and values remain semantically aligned.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change production runtime fallback behavior or add compatibility seed paths.
- Do not change system setting business defaults unless required to restore column/value alignment.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Follow repository Rust style and backend TDD policy.
- Let migration errors surface directly; no silent fallback or mock success path.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: `Rust workspace`
- **Package manager**: `cargo / just`
- **Test framework**: `cargo test`
- **Build command**: `cargo check`
- **Existing test count**: `not enumerated`

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — no external service required for focused unit test.
- [x] Breaking changes to existing code — migration seed only, impact assessed against table schema.
- [x] Large file generation — not applicable.
- [x] Long-running tests — repository has `just test` timeout wrapper.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Updated migration seed implementation.
- Focused Rust unit test for seed column/value alignment.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Unit test fails before production fix and passes after.
- [ ] Rust formatting passes.
- [ ] Relevant Rust validation passes.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
just test
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
