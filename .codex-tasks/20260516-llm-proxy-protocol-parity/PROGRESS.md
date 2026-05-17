# Progress Log

## 2026-05-16

- Expanded `TokenUsage` and request record storage/API/frontend fields for modality tokens, reasoning tokens, Claude cache 5m/1h split, usage source, and usage semantic.
- Updated local Postgres `request_records` and `request_candidates` with additive `ADD COLUMN IF NOT EXISTS` schema changes; did not run destructive baseline migration.
- Enhanced OpenAI/Responses/Claude/Gemini usage parsing from real provider fields only. No token estimation fallback was added.
- Fixed OpenAI stream conversion lifecycle so finish chunks wait for usage-only chunks, and EOF flushes pending terminal events when usage is absent.
- Added Realtime/WebSocket `response.done.response.usage` parsing and writes parsed usage to successful request records.
- Added endpoint metadata registry for default path, model/stream body policy, auth scheme, upstream stream policy, and stream usage policy; compact Responses is treated as force-non-stream for streaming routing.
- Restarted local backend on `127.0.0.1:5555` and verified the provided streaming curl succeeds with `HTTP_STATUS:200` and a final usage-only chunk.
- Verified DB latest record `019e2fa4-8bc9-70d2-848a-b0c7054fc17d` is `success/settled` with `prompt_tokens=27`, `completion_tokens=1779`, `total_tokens=1806`, `total_cost=0.05350500`, and `finished_at` set.
- Verified admin list and active request-record APIs return the latest record as `success/settled` with usage and cost fields.

Validation:
- `cargo fmt --all --check`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p proxy format_conversion -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy:: -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_records -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_candidates -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_housekeeping -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test --workspace --lib --bins --tests`
- `pnpm lint:frontend`

## 2026-05-16 Phase 2 Start

- User approved starting implementation for broader Aether/new-api alignment.
- Phase 2 begins with endpoint/data-format separation, then richer chat canonical conversion, then endpoint-specific usage validation.
- Preserve current debug-first behavior: no token estimation fallback and no silent conversion loss.

## 2026-05-16 Phase 2 Endpoint/Data Format

- Added endpoint metadata fields for endpoint family, endpoint kind, and conversion data format.
- Added OpenAI completions/images/embeddings/audio/moderations/realtime, Gemini embeddings/video, and rerank endpoint metadata with default upstream paths.
- Candidate matching now treats equal data format as pass-through and rejects non-chat cross-format conversion, so image/audio/embedding endpoints cannot enter chat normalizers.
- Added local routes for JSON-capable OpenAI non-chat endpoints and Gemini embedding endpoints; multipart-only body handling remains explicitly unsupported by the current JSON proxy boundary.

Validation:
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy::formats -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy::candidate -- --nocapture`

## 2026-05-16 Phase 2 Canonical Request Conversion

- Expanded the chat canonical request model across OpenAI Chat, OpenAI Responses, Claude Messages, and Gemini to carry tools, tool choices, tool calls, tool results, image/file/audio blocks, top_p, stop sequences, response_format, reasoning effort, and thinking budget.
- Removed the old explicit “tools unsupported” request conversion path; conversion now preserves tool definitions and tool/result blocks where target formats can represent them.
- OpenAI image data URLs are parsed into canonical media blocks and emitted back as data URLs when converting to OpenAI formats; Gemini outputs official camelCase request fields for upstream compatibility.
- Split large request normalizers into smaller request/request_codec/request_tools modules so request entrypoints stay under the project file-size limits.

Validation:
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p proxy format_conversion -- --nocapture`

## 2026-05-16 Phase 2 Endpoint Usage

- Split usage extraction into OpenAI, Claude, Gemini, Rerank, and common parser modules.
- OpenAI-compatible endpoints now parse real `usage` from Chat, Completions, Responses, Embeddings, Images, Audio, and Moderations when upstream provides token fields.
- Gemini embedding endpoints now parse real `usageMetadata` using the same thinking/tool/modality-aware parser as Gemini chat.
- Rerank endpoints now parse standard `usage` and provider `meta.tokens` shapes without estimating tokens.
- Responses with no usage remain visible as `missing_usage`; no token estimation fallback was added.

Validation:
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy::proxy::usage -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p proxy format_conversion -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy::formats -- --nocapture`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy::candidate -- --nocapture`
- `cargo fmt --all --check`

## 2026-05-16 Phase 2 End-to-End Validation

- Re-ran the provided OpenAI Chat streaming curl against `127.0.0.1:5555`; response completed with `HTTP_STATUS:200`, `[DONE]`, and the final usage-only chunk `prompt_tokens=27`, `completion_tokens=1684`, `total_tokens=1711`.
- Verified the latest DB request record `019e2ff3-0f64-7933-aa2d-b7aaace251d4` is `success/settled`, has `usage_source=openai`, `usage_semantic=openai`, `total_cost=0.05065500`, and has `finished_at` set.
- Verified admin request list returns the same latest request as `success/settled` with token and cost fields.
- Verified admin active-record API for that request id returns the completed record instead of leaving it pending.

Validation:
- `curl --request POST http://127.0.0.1:5555/v1/chat/completions ... stream=true`
- `PGPASSWORD=123456 /opt/homebrew/Cellar/libpq/18.4/bin/psql -h localhost -p 5433 -U postgres -d postgres ... request_records`
- `curl http://127.0.0.1:5555/api/admin/request-records?skip=0&limit=3`
- `curl http://127.0.0.1:5555/api/admin/request-records/active`
