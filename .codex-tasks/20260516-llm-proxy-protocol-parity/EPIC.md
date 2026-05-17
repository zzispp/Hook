# LLM Proxy Protocol Parity Epic

Goal: align Hook LLM proxy with the strongest relevant parts of Aether and new-api while preserving Hook debug-first semantics.

Scope:
- Endpoint/protocol metadata for OpenAI, Claude, Gemini request routing.
- Streaming lifecycle fixes: usage-only frames, completed usage, explicit stream end reason, timeout semantics.
- Usage schema and billing propagation for OpenAI, Claude, Gemini, and Realtime.
- No silent estimation or mock success. Missing upstream usage remains visible unless a real upstream usage event is parsed.
