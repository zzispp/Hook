# Progress

- Started: 2026-05-12
- Status: Done
- Scope: Persist and display request token usage.

## Completed

- Persisted prompt, completion, total, cache creation, and cache read token counts on request candidates.
- Exposed token fields through request record list/detail DTOs and frontend types.
- Updated the request records Tokens column to show input/output on the first line and cache creation/cache read on the second line.
- Verified real OpenAI Chat and Gemini native requests write token counts to local DB.
- Verified real converted request `openai_chat -> openai_cli` writes token counts to local DB.

## Validation

- `cargo check -p backend`
- `cargo test -p storage --test provider_request_records`
- `cargo test -p storage --test provider_request_candidates`
- `pnpm lint:frontend`
- `pnpm build:frontend`
