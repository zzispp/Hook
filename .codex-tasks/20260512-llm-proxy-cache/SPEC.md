# LLM Proxy Redis Cache

## Goal

Move LLM proxy request-time policy reads from database hot paths to Redis caches while preserving current scheduling, retry, failover, timeout, billing, and format-conversion semantics.

## Scope

- Cache token authentication policy by token hash.
- Cache scheduling policy as a full Redis snapshot covering system scheduling mode, global models, billing groups, providers, endpoints, provider keys, and provider model bindings.
- Rebuild or invalidate Redis after relevant CUD operations.
- Keep request audit records and token usage writes in the database.
- Avoid database deadlocks by running cache rebuilds only after database mutations complete and by never holding a database transaction while waiting on Redis.

## Validation

- `timeout 60s cargo check -p backend -p proxy`
- Targeted Rust tests where existing test structure allows.

