# Progress

## 2026-05-19

- Confirmed live request returns `user is disabled or unavailable`.
- Confirmed database token, user, and wallet state are valid.
- Confirmed Redis scheduling snapshot does not include the newly registered user.
- Identified code gap: `ProxyCachedUserUseCase::sign_up` does not refresh proxy scheduling snapshot.

## Snapshot Map

- `SchedulingSnapshot` settings are loaded through `SettingStore::get_system_settings`: `default_rate_limit_rpm`, `scheduling_mode`, client/provider request-record body/header policies, and `provider_cooldown_policy`.
- `models` are loaded from `global_models`: id/name/active state and default pricing. Runtime use: model visibility, route matching, billing defaults, model listing.
- `groups` are loaded from `billing_groups` plus group model/provider bindings through `GroupStore::list_groups`. Runtime use: token group lookup, allowed model/provider checks, billing multiplier.
- `users` are loaded from `users` plus configured system users. Runtime use: token owner availability, active status, user model/provider access, quota mode, user rate limits.
- `providers` are loaded from active providers plus `provider_endpoints`, `provider_api_keys`, and `provider_models`. Runtime use: provider/model candidate selection, endpoint/key eligibility, upstream routing, provider retries/timeouts/conversion, pricing overrides.

## CUD Audit

- User writes: `UserRepository::create`, `replace`, and `delete` mutate snapshot-owned user fields. `UserUseCase::sign_up` calls `repository.create` internally, so use-case wrappers can miss this path. `record_login` and password reset do not mutate snapshot-owned fields. User delete also removes API tokens and must bump proxy auth cache.
- API token writes: `ApiTokenRepository::create_token`, `update_token`, `update_any_token`, `delete_token`, `delete_any_token`, and `delete_expired_tokens` mutate proxy auth cache inputs, not scheduling snapshot inputs.
- Setting writes: `SettingRepository::update_system_settings` can mutate snapshot-owned rate limit, scheduling, request-record, and provider cooldown policy fields. The update payload is broad, so rebuilding scheduling on any settings update is the direct explicit behavior.
- Model writes: `ModelRepository::create_global_model`, `update_global_model`, and `delete_global_model` mutate snapshot models. Batch delete calls `delete_global_model` for each id, so repository-level invalidation covers it.
- Group writes: `GroupRepository::create_group`, `update_group`, and `delete_group` mutate snapshot group data and group model/provider bindings.
- Provider writes: provider, endpoint, API key, and model-binding create/update/delete methods mutate snapshot provider data. `release_provider_cooldown` does not mutate scheduling snapshot, but it must clear the proxy provider cooldown cache.

## Invalidation Layer Decision

- Do not put `LlmProxyCache` or Redis invalidation into `crates/storage`. That crate is the generic persistence layer and should not depend on backend-specific proxy cache behavior.
- Move invalidation below use-case wrappers into backend repository adapters: `CachedUserRepository`, `CachedApiTokenRepository`, `CachedSettingRepository`, `CachedModelRepository`, `CachedGroupRepository`, and `CachedProviderRepository`.
- This layer is low enough to catch service-internal writes such as `sign_up -> repository.create`, while keeping cache side effects in the backend composition boundary.

## Implementation Notes

- Replaced proxy cache use-case wrappers with repository adapters in `apps/hook_backend/src/proxy_cache_hooks/`.
- Updated `apps/hook_backend/src/startup.rs` to inject cached repositories into services instead of wrapping services.
- Added regression coverage for `sign_up` proving repository `create` refreshes the scheduling snapshot.
- Targeted validation passed: `cargo test -p backend proxy_cache_hooks::user::user_tests -- --nocapture`.
- Compile validation passed: `cargo check -p backend`.
- Final validation passed: `just format`, targeted regression, `just check`, and `just test`.

## Scope Change

- User requested a broader audit: inspect Redis snapshot contents, trace every CUD path for those data sources, and evaluate moving cache rebuild/invalidation to the lowest repository layer.

## Recovery

任务: Audit LLM proxy Redis scheduling snapshot invalidation coverage and implement the root fix.
形态: single-full
进度: 6/6
当前: Complete.
文件: `.codex-tasks/20260519-signup-refresh-proxy-snapshot/TODO.csv`
下一步: None.
