# Progress Log

## Session Start

- **Date**: 2026-06-03
- **Task name**: `image-generation-aether-alignment`
- **Task dir**: `.codex-tasks/20260603-image-generation-aether-alignment/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust / Hook backend proxy routing

## Context Recovery Block

- **Current milestone**: #1 — Map Hook image routing pipeline
- **Current status**: IN_PROGRESS
- **Last completed**: none
- **Current artifact**: `apps/hook_backend/src/llm_proxy/**`
- **Key context**: Aether only treats `tool_choice.type = image_generation` as explicit image intent. It does not remove image tools and does not treat a `tools` declaration alone as intent.
- **Next action**: Inspect Hook proxy request, candidate selection, body conversion, and finalize paths.

---

<!-- Append entries below as each milestone completes -->

## 2026-06-03 Completion

- Mapped Hook routing: `CandidateRequest.api_format` was used for both client trace and endpoint matching, so OpenAI Chat/Responses requests with image intent could not select `openai:image`.
- Added `routing_api_format` while preserving `client_api_format` in traces and records.
- Aligned intent semantics with Aether: only `tool_choice: "image_generation"` or `tool_choice: {"type":"image_generation"}` routes to image generation; `tools: [{"type":"image_generation"}]` alone does not.
- Added OpenAI Chat/Responses to OpenAI Image request bridge without removing image tools.
- Added sync response bridge for OpenAI Image back to OpenAI Responses passthrough and OpenAI Chat markdown image completion.
- Validation passed:
  - `timeout 60 cargo test -p backend image_generation_intent --no-fail-fast`
  - `timeout 60 cargo test -p backend chat_image_bridge_preserves_image_tool_declaration --no-fail-fast`
  - `timeout 60 cargo test -p backend matching_candidate_parts --no-fail-fast`
  - `timeout 60 cargo test -p backend image_bridge_response --no-fail-fast`
  - `timeout 60 cargo test -p backend request::tests --no-fail-fast`
  - `timeout 60 cargo test -p backend image_generation --no-fail-fast`
