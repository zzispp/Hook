# Stream Conversion Timing Alignment

## Goal

Align Hook proxy streaming format conversion timing with aether: upstream streaming responses must be parsed, converted, and emitted incrementally instead of buffering the full upstream stream before converting.

## Scope

- Keep request conversion timing unchanged: convert before sending each upstream attempt.
- Keep non-stream response conversion timing unchanged: convert after the upstream JSON response is read.
- Change stream response conversion to use incremental chunk parsing and conversion.
- Preserve provider timeout, request record first-byte/success/failure recording, and retry/failover semantics.

## Evidence

- aether converts request bodies before upstream attempts in `src/api/handlers/base/chat_handler_base.py`.
- aether converts non-stream responses after reading JSON in `src/api/handlers/base/chat_sync_executor.py`.
- aether converts streaming events per chunk in `src/api/handlers/base/stream_processor.py`.
- Hook currently buffers stream responses through `response.bytes().await` in `apps/hook_backend/src/llm_proxy/proxy/transport_read.rs`.
