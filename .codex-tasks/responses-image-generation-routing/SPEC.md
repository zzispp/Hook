# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Fix Responses requests whose `input` contains image/tool history items from being routed to conversion endpoints that cannot preserve those items.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not implement semantic conversion for `image_generation_call`.
- Do not change production containers or database state.
- Do not treat `tools: [{type:"image_generation"}]` alone as image-generation intent.

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

- [x] External dependencies (APIs, services) — Brave search result checked for OpenAI image tool semantics.
- [x] Breaking changes to existing code — impact is candidate filtering for unsupported conversion only.
- [x] Large file generation — no large generated files.
- [x] Long-running tests — use 60-second timeout for backend tests.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Request feature detection for non-convertible OpenAI Responses input items.
- Candidate matching excludes conversion endpoints for those requests.
- Regression tests for `image_generation_call` input history.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] `image_generation_call` in Responses `input` is detected as non-convertible.
- [ ] Candidate matching keeps exact `openai:cli` endpoint and excludes `openai:chat` conversion.
- [ ] Existing custom tool conversion exclusion still passes.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
timeout 60 cargo test -p hook_backend responses_custom_tool responses_image_generation_call && timeout 60 cargo test -p hook_backend llm_proxy::candidate::selection::tests::matching && timeout 60 cargo check -p hook_backend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
