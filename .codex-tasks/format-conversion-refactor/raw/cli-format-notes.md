# CLI Format Notes

## OpenAI / Codex Responses

Sources:

- `/Users/bubu/Downloads/cli/codex-main/codex-rs/protocol/src/models.rs`
- `/Users/bubu/Downloads/cli/codex-main/codex-rs/core/src/tools/context.rs`
- `/Users/bubu/Downloads/cli/codex-main/codex-rs/core/src/tools/context_tests.rs`

Facts:

- Codex uses Responses input items as the canonical OpenAI CLI wire shape.
- `message.content` is a `ContentItem[]` with `input_text`, `input_image`, and `output_text`.
- `function_call` has `call_id`, `name`, and string `arguments`.
- `function_call_output.output` is not string-only. It is a union:
  - string text
  - array of structured items: `input_text`, `input_image`, `encrypted_content`
- `custom_tool_call` has `call_id`, `name`, and string `input`.
- `custom_tool_call_output.output` uses the same union encoding as `function_call_output.output`.
- `tool_search_call` and `tool_search_output` are official Codex Responses item types. They do not have a neutral equivalent in the current internal model yet.
- Codex's text derivation from function output content items includes non-blank `input_text` items joined by `\n`; image and encrypted items are ignored only for lossy text previews, while the structured content remains authoritative.

## Claude Messages

Sources:

- `/Users/bubu/Downloads/cli/claude-code-main/src/remote/sdkMessageAdapter.ts`
- `/Users/bubu/Downloads/cli/claude-code-main/src/services/api/claude.ts`

Facts:

- Tool calls use assistant `tool_use` content blocks.
- Tool results use user `tool_result` content blocks.
- Thinking is represented by `thinking` blocks with a `signature` when present.
- Tool result blocks are detected by content shape, not by an external parent id.

## Gemini

Sources:

- `/Users/bubu/Downloads/cli/gemini-cli-main/packages/sdk/src/session.ts`
- `/Users/bubu/Downloads/cli/gemini-cli-main/packages/core/src/core/geminiChat.ts`
- `/Users/bubu/Downloads/cli/gemini-cli-main/packages/core/src/config/config.ts`

Facts:

- Gemini request turns are `contents[].parts[]`.
- Tool calls use `functionCall`; tool results use `functionResponse`.
- Multimodal content uses `inlineData` or `fileData`.
- Thinking uses text parts with `thought: true` and can carry `thoughtSignature`.
- Switching auth modes can require stripping `thoughtSignature`, so signatures must be treated as provider-bound data, not generic text.

## Implementation Plan

- Keep the existing registry pipeline: source format -> `InternalRequest/InternalResponse/InternalStreamEvent` -> target format.
- Repair OpenAI Responses request parsing first because it currently silently converts structured `function_call_output.output` arrays into empty text.
- Preserve structured text/image tool output content as internal `ToolResult.content` blocks.
- Emit explicit `UnsupportedContent` for official item/block shapes that the current internal model cannot represent without loss.
- Add focused tests before implementation and validate with `cargo test -p proxy format_conversion -- --nocapture` under a 60-second timeout wrapper.
