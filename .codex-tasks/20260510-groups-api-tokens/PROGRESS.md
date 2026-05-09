# Progress

## 2026-05-10

- Started implementation task.
- User clarified token defaults: unlimited quota, all models, rate follows future system setting.
- Reviewed existing patterns: domain crates expose Axum routers, application traits, storage adapters, and DTOs in `types`.
- Chosen backend boundaries: `group` crate for billing groups and `api_token` crate for user API tokens.
- Backend group/token crates, storage, migration, and startup routing were implemented.
- `cargo check --workspace` passed before frontend work, then formatting was applied.
- Added `token_type` and `request_count` to API tokens, plus follow-system rate semantics where `0` is stored for default/system rate behavior.
- Added admin token routes under `/api/admin/tokens` for managing all tokens; independent tokens are owned by the creating admin, and user tokens are owned by the selected user.
- Added follow-up migrations `000003` and `000004` because `000002` had already been applied locally. `cargo run -p backend -- migration up` applied both successfully.
- Reworked user token management to show key name, copyable key, cost CNY, request count, status, last used, and edit/enable-disable/delete actions.
- Added admin token management page and menu/API permissions, plus split group/token frontend files to stay within size limits.
- Validation passed: `cargo fmt --all`, `pnpm lint:frontend`, `cargo check --workspace`, `cargo run -p backend -- migration up`, and `cargo test --workspace` with a 60-second alarm wrapper. `just check` could not run because `just` is not installed in this environment.
- Added system-group default behavior for token creation: omitted or blank `group_code` resolves to the built-in `default` billing group, while updates still validate explicit group codes.
- Added migration `000005` to mark the default group as the system group, update its description, and bind `groups_available_read` to the user model catalog menu.
- Split model drawer pricing into focused components and added group-adjusted pricing in the user model detail drawer.
- Added api token validation tests for omitted and blank `group_code` resolving to the system group.
- Incremental validation passed again: `cargo fmt --all`, `cargo check --workspace`, `pnpm lint:frontend`, `cargo run -p backend -- migration up`, and `cargo test --workspace` with a 60-second alarm wrapper.
- Localized group/token frontend surfaces: billing group table headers, system group name/description, token/group delete confirmations, and empty states in token create selects.
- Validation passed for localization update: locale JSON parse check, `pnpm lint:frontend`, and `cargo check --workspace`.
- Added billing group model bindings through `billing_group_models`, exposed `allowed_model_ids` on group DTOs, and validated that token model restrictions cannot exceed the selected billing group.
- Updated the admin billing group modal to select allowed global models; an empty model selection means all models. The group table now shows all-model vs selected-model count, and token model options are narrowed by selected group.
- Final validation passed: `cargo fmt --all`, `cargo check --workspace`, `pnpm lint:frontend`, `cargo run -p backend -- migration status`, and `cargo test --workspace` with a 60-second alarm wrapper. Migration status shows `m20260510_000006_create_billing_group_model_bindings` applied.
