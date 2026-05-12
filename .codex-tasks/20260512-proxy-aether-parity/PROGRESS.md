# Progress

## 2026-05-12

- Started proxy parity task. Initial known gaps: one-shot candidate selection, no failover/retry, no health/quota/time-window/capability/api-format key filtering, no cache affinity/RPM gating, flat endpoint api_format semantics, and unused scheduler crate.
- Renamed backend proxy app module from `openai` to `llm_proxy` and removed the old `apps/hook_backend/src/openai` path.
- Implemented scheduler-backed candidate ordering across the LLM proxy path with full active provider collection, billing group filtering, token/group model checks, `FixedOrder`, `CacheAffinity`, and `LoadBalance`.
- Preserved request/response/stream format conversion. Conversion candidates remain gated by provider/endpoint conversion settings and are demoted by scheduler priority unless the provider keeps conversion priority.
- Implemented JSON proxy retry/failover across candidates and realtime websocket upstream connection retry/failover. Established websocket relays record failures but cannot switch providers after upgrade.
- Removed provider UI displays and form fields for probing RPM, cache TTL, probe interval, non-stream timeout, and stream first-byte timeout.
- Validation passed: `cargo check -p backend`, `cargo test -p proxy scheduler -- --nocapture`, `cargo test -p proxy -- --nocapture`, `cargo test -p storage --test provider_request_records -- --nocapture`, and `pnpm lint:frontend`.
