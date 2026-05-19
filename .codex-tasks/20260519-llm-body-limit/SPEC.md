# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Fix LLM proxy request rejection caused by Axum's default 2MB body limit on `Json<Value>` extractors.
- Keep the fix explicit at the LLM proxy route boundary.
- Add a regression test that proves oversized JSON reaches proxy logic instead of being rejected by the extractor.

## Non-Goals

- Do not change Nginx configuration.
- Do not repurpose request-record truncation settings as ingress request size settings.
- Do not add fallback or mock success behavior.

## Constraints

- Follow backend `AGENTS.md`.
- Use TDD for backend changes.
- Run `cargo fmt --all` after Rust edits.
- Backend unit test commands must use a 60 second timeout.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024, Axum 0.8.9
- **Package manager**: Cargo
- **Test framework**: Cargo tests
- **Build command**: `cargo check`

## Risk Assessment

- [x] Breaking changes to existing code — limit applies only to LLM proxy routers.
- [x] Long-running tests — targeted commands use `timeout 60`.

## Deliverables

- Regression test under `apps/hook_backend/src/llm_proxy`.
- Route-level body limit configuration for `/v1` and `/v1beta` proxy routers.

## Done-When

- [ ] Oversized JSON route-limit test fails before the fix and passes after the fix.
- [ ] `cargo fmt --all` has been run.
- [ ] Targeted backend validation passes.

## Final Validation Command

```bash
cargo fmt --all && timeout 60 cargo test -p backend llm_proxy_body_limit -- --nocapture
```
