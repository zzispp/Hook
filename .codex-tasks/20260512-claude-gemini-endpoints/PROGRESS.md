# Progress

- Started: 2026-05-12
- Status: Done
- Scope: Add Claude and Gemini client-compatible public endpoints.

## Implementation

- Added `/v1/messages` as Claude client format entrypoint.
- Added `/v1beta/models/{model}:{action}` as Gemini client format entrypoint.
- Added `/v1beta/*` auth whitelist and frontend proxy rewrite.
- Kept format semantics explicit: request path determines client format; selected provider endpoint format determines upstream format.
- Added Claude and Gemini upstream auth headers and Gemini `alt=sse` stream URL handling.
- Preserved Gemini native stream semantics by reading the route action into internal `stream` state while removing `stream` from Gemini upstream bodies.

## Validation

- `cargo check -p backend`: passed.
- `cargo test -p proxy format_conversion`: passed, 6 tests.
- `just check`: passed.
- `pnpm lint:frontend`: passed.
- Real calls passed:
  - `/v1/messages` with `gpt-5.5`: 200.
  - `/v1beta/models/gpt-5.5:generateContent`: 200.
  - `/v1beta/models/gpt-5.5:streamGenerateContent`: 200.
  - `/v1/messages` with `claude-opus-4-7`: 200, `needs_conversion=false`.
  - `/v1beta/models/gemini-3.1-pro-preview:generateContent`: 200, `needs_conversion=false`.
  - `8082 /v1beta/models/gemini-3.1-pro-preview:generateContent`: 200.
