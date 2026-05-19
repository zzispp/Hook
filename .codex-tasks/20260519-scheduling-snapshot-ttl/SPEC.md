# Scheduling Snapshot TTL

## Goal

Add an explicit configurable TTL for the LLM proxy scheduling snapshot cache while keeping repository-level cache refresh as the primary correctness path.

## Scope

- Add Redis config for `scheduling_snapshot_ttl_seconds`.
- Write scheduling snapshots with `SETEX` only when the configured TTL is greater than zero.
- Keep startup warm rebuild and repository CUD invalidation unchanged.
- Validate configuration and cache command behavior with automated checks.
