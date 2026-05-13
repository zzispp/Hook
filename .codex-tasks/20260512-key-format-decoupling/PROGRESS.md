# Progress

- Started: 2026-05-12
- Status: Done
- Scope: Remove API format binding from provider API keys and keep conversion keyed by endpoint only.

## Validation

- `cargo check -p backend` passed.
- `just check` passed.
- `cargo test -p proxy scheduler` passed under a 60 second alarm.
- `cargo test -p storage provider` passed under a 60 second alarm.
- `cargo test -p storage request_record` passed under a 60 second alarm.
- `pnpm lint:frontend` passed.
- `pnpm build:frontend` passed with the existing `Axios error: unauthorized` prerender log and exit 0.
- Local DB column `provider_api_keys.api_formats` was dropped.
- Real HTTP tests against `127.0.0.1:5555` returned 200 for OpenAI chat non-stream, chat stream, responses, and responses compact.
- Forced conversion test returned 200 with request trace `openai_chat -> openai_cli` and `needs_conversion = true`.
- WebSocket `/v1/realtime?model=gpt-5.5` reached backend selection/audit and returned 502 because upstream realtime responded 404.
