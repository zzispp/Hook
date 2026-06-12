# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Align Hook with Aether for OpenAI Responses `image_generation_call` request history conversion.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not make `custom_tool_call` semantically convert to chat formats.
- Do not change production state.
- Do not broaden image generation intent from `tool_choice` to the `tools` array.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

-

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024 workspace
- **Package manager**: cargo / just
- **Test framework**: Rust `cargo test`
- **Build command**: `cargo check -p hook_backend`
- **Existing test count**: repository-dependent

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — Aether source inspected locally.
- [x] Breaking changes to existing code — scope limited to image_generation_call conversion eligibility.
- [x] Large file generation — none.
- [x] Long-running tests — use 60-second timeout for backend-focused commands.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Allow `image_generation_call` in OpenAI Responses request conversion whitelist.
- Keep `custom_tool_call` and `custom_tool_call_output` as non-convertible.
- Route Responses requests with image history through conversion when otherwise compatible.
- Tests proving image history converts to OpenAI Chat/Claude, and custom tools remain blocked.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] `image_generation_call` Responses input converts through proxy registry.
- [ ] Candidate matching allows conversion for image history but not custom tool history.
- [ ] Existing explicit image intent routing remains unchanged.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
cargo fmt --check && timeout 60 cargo test -p proxy image_generation_call && timeout 60 cargo test -p hook_backend responses_non_convertible && timeout 60 cargo test -p hook_backend llm_proxy::candidate::selection::tests::matching && timeout 60 cargo check -p hook_backend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
