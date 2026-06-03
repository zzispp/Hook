# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Fix OpenAI Responses request conversion so `input` items with `type: "reasoning"` do not fail when Hook retries or routes to an OpenAI Chat-compatible upstream.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not silently remove `image_generation` tools from client requests.
- Do not broaden support for unrelated unsupported Responses item types.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Rust workspace in `/Users/bubu/ZwjProjects/Hook`.
- Backend tests use `cargo test`; focused backend unit tests must run with `timeout 60`.
- Follow Debug-First: unsupported capabilities should surface explicit errors instead of silent degradation.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `<auto>`
- **Language/runtime**: `<auto>`
- **Package manager**: `<auto>`
- **Test framework**: `<auto>`
- **Build command**: `<auto>`
- **Existing test count**: `<auto>`

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — not needed for focused conversion tests.
- [x] Breaking changes to existing code — limited to Responses `reasoning` item conversion.
- [x] Large file generation — not applicable.
- [x] Long-running tests — timeout configured.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Code change in conversion layer.
- Focused tests for Responses `reasoning` input conversion.
- Analysis-backed answer for image generation handling.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Responses `reasoning` input items are mapped into canonical thinking content and emitted to OpenAI Chat as explicit reasoning fields.
- [ ] Existing unsupported Responses request items still fail clearly.
- [ ] Image generation requests are not silently downgraded by deleting tools.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
timeout 60 cargo test -p proxy format_conversion
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Run focused conversion tests.
2. Confirm `reasoning` item no longer produces `unsupported input item type reasoning`.
3. Confirm unrelated unsupported item tests still fail clearly.
