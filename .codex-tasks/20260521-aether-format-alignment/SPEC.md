# Aether Format Alignment

## Goal

Align Hook Chat and CLI endpoint format conversion with Aether canonical formats:

- `openai:chat`
- `openai:cli`
- `openai:compact`
- `claude:chat`
- `claude:cli`
- `gemini:chat`
- `gemini:cli`

## Scope

- Format conversion must use `source -> Internal -> target`.
- Canonical `family:kind` IDs are the business truth.
- OpenAI Chat, OpenAI Responses, Claude, and Gemini request/response/stream/error conversion should preserve equivalent fields where representable.
- Backend endpoint metadata and frontend format options must expose canonical IDs.

## Out Of Scope

- Video, image generation, embedding, audio, realtime, and rerank cross-format expansion.
- Old snake_case compatibility or data migrations.

## Validation

- `timeout 60 cargo test -p proxy`
- `timeout 60 cargo test -p hook_backend llm_proxy`
- `just test`
- `pnpm lint:frontend`
- `pnpm build:frontend`
