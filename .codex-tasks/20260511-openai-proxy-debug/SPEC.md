# OpenAI Proxy Debug

## Goal

Diagnose why the local OpenAI-compatible chat completions call fails, repair the root cause using the current Hook codebase and the aether reference implementation where relevant, then verify non-streaming, streaming, and websocket OpenAI-compatible calls.

## Boundaries

- Do not revert unrelated existing workspace changes.
- Do not add mock success paths or silent fallbacks.
- Keep failures explicit.
- Use the local configured database and provider/model/key records as the source of truth.

## Validation

- Reproduce the current failure.
- Run targeted automated checks where feasible.
- Execute local curl/websocket verification for the three requested endpoint modes.
