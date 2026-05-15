# Progress Log

## Context Recovery Block

- **Current milestone**: complete
- **Current status**: DONE
- **Last completed**: #3 — Validate cold recovery behavior
- **Current artifact**: `TODO.csv`
- **Key context**: pending/processing Redis hashes survive restart, and processing batch now carries a stable batch id. DB writes insert `usage_flush_batches` in the same transaction as usage increments, so post-commit/pre-Redis-clear crashes are idempotent on restart.
- **Known issues**: none for the scoped token/model usage flush recovery path.
- **Next action**: none

## 2026-05-15

- Replaced processing requeue with durable processing retry: restart keeps processing data and batch id, then retries the same batch.
- Added storage `usage_flush_batches` entity/table to record completed token/model usage flush batches.
- Added `record_usage_batch_once` for token/model usage. It skips already-marked batch ids and applies unmarked batches inside one DB transaction with the marker insert.
- Added Redis processing batch id keys for token and model usage processing states.
- Added storage tests proving already-marked batches skip counter updates and new batches write both usage increment and marker.
- Validation passed: `just format`, `cargo check -p backend`, `cargo clippy -p backend --all-targets -- -D warnings`, `just check`, `cargo test -p backend usage_flush -- --nocapture`, `cargo test -p storage --test api_token_usage --test model_usage -- --nocapture`, `just test`.
