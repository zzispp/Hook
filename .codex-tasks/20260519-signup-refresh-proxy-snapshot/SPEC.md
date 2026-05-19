# Goal

Audit the LLM proxy Redis scheduling snapshot from first principles, identify every CUD path that can make the snapshot stale, and decide whether cache rebuild should move down to repository-layer invalidation or stay in application/use-case wrappers.

# Evidence

- Live `curl` to `https://api.hook.rs/v1/chat/completions` returned HTTP 403 with `user is disabled or unavailable`.
- The token hash exists in `api_tokens`, is active, not expired, and is bound to user `019e3f78-63f7-7b71-b1a8-d9192a3101c4`.
- That user exists in `users`, is active, not deleted, and has an active wallet with balance.
- Redis key `hook:llm_proxy:scheduling:snapshot:v2` contains only the system admin user.
- Redis snapshot currently includes settings, global models, billing groups, users, providers, endpoints, provider keys, and provider model bindings.
- `ProxyCachedUserUseCase::sign_up` delegates to the inner service without refreshing the LLM proxy scheduling snapshot, proving at least one stale-cache path exists.

# Boundary

Do not add request-time fallback behavior that silently bypasses the snapshot. Failures should stay visible. Any cache invalidation change must be explicit and testable.
