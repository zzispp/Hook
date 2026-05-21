# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Fix the real OpenAI Responses -> Claude Messages replay failure from request `019e4826`.
- Align Hook with Aether behavior for OpenAI Responses hosted tools: non-function tools such as `web_search` must not require a function `name`.
- Re-run the exact curl body from DB with the user-provided API key and require a real successful response.

## Non-Goals

- Do not add compatibility fallback, mock success, or silently drop unsupported fields.
- Do not broaden unrelated endpoint formats beyond the current Chat/CLI conversion path.

## Constraints

- Keep failures visible.
- Backend tests must use a 60 second hard timeout.
- Preserve existing worktree changes.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace
- **Test framework**: Cargo tests

## Done-When

- [ ] Hosted OpenAI Responses tools parse without requiring `name`.
- [ ] Claude target request includes Aether-equivalent web search tool mapping when web search is requested.
- [ ] Targeted tests pass.
- [ ] Real curl replay with `sk-WHB1pUChuyDWuve4IPNJVaadSjyqyrji` succeeds.

## Final Validation Command

```bash
perl -e 'alarm 60; exec @ARGV' cargo test -p proxy openai_responses && perl -e 'alarm 60; exec @ARGV' cargo test -p backend llm_proxy
```
