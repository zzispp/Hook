# Responses Custom Tool Routing

## Goal

Prevent OpenAI Responses requests containing custom tool items from being routed to provider endpoints that require cross-format conversion.

## Boundary

- Keep same-format `openai:cli` routing unchanged.
- Exclude conversion endpoints such as `openai:chat` and `claude:chat` when the request has Responses custom tool input items.
- Do not add mock conversion, fallback transformation, or silent degradation.

## Validation

- Run targeted backend Rust tests covering candidate matching and request feature detection.
