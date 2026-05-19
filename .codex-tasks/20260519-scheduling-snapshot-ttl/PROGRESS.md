# Progress

## 2026-05-19

- Confirmed existing Redis caches already use TTL for auth token, token usage, provider cooldown, provider cooldown windows, rate-limit buckets, and cache affinity.
- Confirmed only `llm_proxy:scheduling:snapshot:v2` currently writes with plain `SET`.
- Decision: add explicit TTL config as a self-healing safety net. Keep startup refresh and repository-level CUD refresh unchanged.
- Added `redis.scheduling_snapshot_ttl_seconds`; missing field defaults to `0`, which disables snapshot expiration.
- Set local `config/config.yaml` to `3600` seconds so snapshot writes use `SET ... EX 3600`.
- Split cache options and scheduling snapshot write command into focused modules to keep files within project size limits.
- Validation passed: `just format`, `cargo test -p configuration redis_tests::scheduling_snapshot_ttl_defaults_to_disabled -- --nocapture`, `cargo test -p backend llm_proxy::cache::scheduling_snapshot_write::tests -- --nocapture`, `just check`, `just test`.
