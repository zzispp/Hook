# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Verify whether Hook request billing should subtract cached input tokens from normal input billing.
- Compare the behavior with `/Users/bubu/ZwjProjects/aether` and `/Users/bubu/ZwjProjects/new-api`.
- Apply the smallest Hook change that aligns cache-read billing semantics without changing request-record raw usage display.

## Non-Goals

- Do not redesign billing rules, UI layout, or historical request records.
- Do not add fallback billing behavior or mock success paths.

## Constraints

- Follow Hook `AGENTS.md`: Chinese user-facing replies, debug-first, no silent fallback, backend tests under 60 seconds.
- Follow Rust project metrics and keep changes localized.
- Do not edit aether or new-api; read them as references only.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace plus pnpm frontend
- **Package manager**: cargo / pnpm
- **Test framework**: cargo test

## Risk Assessment

- [x] External dependencies — only official OpenAI docs and local reference repos are needed.
- [x] Breaking changes — default billing amount changes for cached input requests only.
- [x] Long-running tests — use 60 second timeout for backend unit tests.

## Deliverables

- Hook billing formula adjustment.
- Focused tests proving cached read tokens are excluded from normal input cost and billed as cache read.
- Concise explanation of request-record display versus actual deduction.

## Done-When

- [ ] aether/new-api evidence is inspected.
- [ ] Hook billing semantics are aligned.
- [ ] Relevant Rust tests pass.

## Final Validation Command

```bash
timeout 60 cargo test -p provider
```
