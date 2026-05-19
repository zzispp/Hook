# Progress

## 2026-05-19

- Started Single Task to align performance monitoring cache hit rate with Aether dashboard token semantics.
- Confirmed current Hook performance monitoring computes cache hits as request count with `cache_read_input_tokens > 0` divided by total request count.
- Confirmed Aether dashboard computes token hit rate as cache read tokens divided by input context tokens, where input context is non-cache input plus cache read.
- Updated aggregation to compute `cache_hit_rate = cache_read_input_tokens / (prompt_tokens + cache_read_input_tokens)`.
- Added a targeted storage test for the new token-based ratio.
- Backend validation passed: `cargo test -p storage --test performance_monitoring`.
- Added a request records table cache hit rate column using the same per-request token formula and `-` for no cache read tokens.
- Added backend i18n seed keys for the new request records column.
- Frontend validation passed: `pnpm lint:frontend`.
