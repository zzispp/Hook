# Progress Log

---

## Session Start

- **Date**: 2026-05-11
- **Task name**: `20260511-provider-proxy-epic`
- **Task dir**: `.codex-tasks/20260511-provider-proxy-epic/`
- **Spec**: See EPIC.md
- **Plan**: See SUBTASKS.csv
- **Environment**: Rust workspace / Next.js frontend / SeaORM

---

## Context Recovery Block

- **Current milestone**: Complete
- **Current status**: DONE
- **Last completed**: #7 — Request candidate audit and observability
- **Current artifact**: `.codex-tasks/20260511-provider-proxy-epic/SUBTASKS.csv`
- **Key context**: User approved the provider management and group-scoped proxy plan. Scope includes OpenAI/Gemini/Claude mutual conversion, endpoint conversion policy, header/body rewrite rules, and billing group provider binding.
- **Known issues**: None yet.
- **Next action**: None.

---

## Milestone 1: Provider storage schema and domain types

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What is being done**:
  - Add provider-related tables and identifiers.
  - Add storage entities and public types.
  - Keep existing `models` table as provider model binding.
- **What was done**:
  - Added `providers`, `provider_endpoints`, `provider_api_keys`, `provider_models`, `billing_group_providers`, and `request_candidates` baseline tables.
  - Removed the old `models` table path and moved model binding semantics to `provider_models`.
  - Added `types::provider` and `storage::provider` foundations.
- **Validation**: `just check` -> exit 0
- **Next step**: Milestone 2 — Provider admin backend API

## Milestone 2: Provider admin backend API

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Added `crates/provider` with application ports, service, validation, storage infra, and admin API routes.
  - Registered Provider API in backend startup.
  - Added explicit rejecting secret cipher so upstream keys cannot be persisted before encryption is configured.
- **Validation**: `just check` -> exit 0
- **Next step**: Milestone 3 — Billing group provider binding

## Milestone 3: Billing group provider binding

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Added `allowed_provider_ids` to billing group domain, create/update payloads, and responses.
  - Persisted group-provider bindings through `billing_group_providers`.
  - Added provider existence validation before group create/update.
  - Registered `StorageGroupProviderCatalog` in backend startup.
- **Validation**: `just check` -> exit 0
- **Next step**: Milestone 4 — Provider admin frontend UI

## Milestone 4: Provider admin frontend UI

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Added provider frontend types and API actions.
  - Added `/dashboard/admin/providers` management page with provider CRUD and binding panels for endpoints, keys, and model ids.
  - Added group provider selection to billing group forms and table.
  - Added default RBAC menu/API bindings and backend i18n seed keys for provider management.
- **Validation**:
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
- **Next step**: Milestone 5 — API format registry and conversation conversion

## Milestone 5: API format registry and conversation conversion

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Added `crates/proxy` with a hub-and-spoke format conversion registry.
  - Added OpenAI Chat, Gemini Chat, and Claude Chat normalizers for text conversation requests, responses, and stream events.
  - Added explicit unsupported-feature errors for first-phase non-text/tool paths instead of silent degradation.
  - Added behavior tests for request, response, stream conversion, and unsupported tool rejection.
- **Validation**:
  - `cargo test -p proxy format_conversion` -> exit 0
  - `just check` -> exit 0
  - `timeout 60 cargo test -p proxy format_conversion` could not be executed because this shell does not provide `timeout`.
- **Next step**: Milestone 6 — Group-scoped proxy scheduler and failover

## Milestone 6: Group-scoped proxy scheduler and failover

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Added `proxy::scheduler` with pure candidate-building types for group/token scoped routing.
  - Added provider, endpoint, key, and provider-model filtering.
  - Added fixed order, cache affinity, and load-balance ordering behavior.
  - Added explicit failover executor that advances after retryable failures and stops on success or fatal failure.
  - Added tests for group provider filtering, no-model availability error, conversion demotion, cache affinity, load-balance stability, and failover.
- **Validation**:
  - `cargo test -p proxy` -> exit 0
  - `just check` -> exit 0
  - `timeout 60 cargo test -p proxy` could not be executed because this shell does not provide `timeout`.
- **Next step**: Milestone 7 — Request candidate audit and observability

## Milestone 7: Request candidate audit and observability

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Added `CandidateAuditRecorder` for available, attempted, success, failure, and no-candidate audit records.
  - Added storage input type and `ProviderStore` APIs to create and list `request_candidates`.
  - Added proxy audit tests for success, failure, and no-candidate cases.
  - Added storage tests using SeaORM mock DB to verify create/list request candidate behavior.
- **Validation**:
  - `cargo test -p proxy request_candidate` -> exit 0
  - `cargo test -p storage request_candidate` -> exit 0
  - `just check` -> exit 0
  - `timeout 60 cargo test -p proxy request_candidate` could not be executed because this shell does not provide `timeout`.
- **Next step**: Milestone 8 — Full validation pass

## Milestone 8: Full validation pass

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Re-ran backend workspace check.
  - Re-ran frontend lint.
  - Re-ran frontend production build.
- **Validation**:
  - `just check` -> exit 0
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.
- **Next step**: None

## Milestone 9: Provider type dropdown alignment

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **Scope adjustment**: User clarified the first phase only needs a single select option, `providerTypeCustom`.
- **What was done**:
  - Keep provider type as a select instead of free text.
  - Limit frontend provider type options to `custom`.
  - Keep only `providers.providerTypeCustom` in backend i18n seed.
  - Limit backend `provider_type` validation to `custom`.
- **Validation**:
  - `just check` -> exit 0
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 10: Provider list scheduling strategy

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Move scheduling mode semantics from individual providers to the provider list/system setting level.
  - Remove scheduling mode from provider create/edit and provider table rows.
  - Add a provider list toolbar setting for cache affinity, load balance, and fixed order.
  - Update scheduler input so candidate ordering uses the list-level scheduling mode.
- **Validation**:
  - `cargo test -p proxy scheduler` -> exit 0
  - `just check` -> exit 0
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 11: Provider bindings modal

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **What was done**:
  - Remove the always-visible right-side provider bindings panel.
  - Open endpoint, key, and model binding management in a modal after selecting a provider from the table.
  - Keep provider create/edit/delete actions available from the table row actions.
- **Validation**:
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 12: Request records admin UI

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **Scope**:
  - Add request-level aggregation over existing request candidate audit records.
  - Add admin list API and detail trace API.
  - Add admin request records page with auto refresh and drawer detail.
  - Keep cost fields as explicit zero placeholders for now; do not add real cost settlement.
- **What was done**:
  - Added request record list/detail types and provider admin APIs.
  - Aggregated request-level records from `request_candidates`, including pending, streaming, success, and failed status.
  - Added admin request records navigation, permissions, backend i18n seed entries, list page, auto refresh, and drawer trace detail.
  - Added storage tests for request aggregation and trace detail.
  - Fixed `RequestRecordListRequest::default()` so Rust callers get the same default limit as query deserialization.
- **Validation**:
  - `cargo test -p storage request_record` -> exit 0
  - `just check` -> exit 0
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Frontend build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 16: Provider endpoint dialog alignment

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **Scope**:
  - Align the add endpoint modal with Aether.
  - Add searchable API format selection.
  - Show Base URL and custom path placeholders.
  - Show the default path helper text for each API format.
- **What was done**:
  - Updated the endpoint add modal title, provider-specific description, and footer actions.
  - Replaced the plain API format select with a searchable autocomplete.
  - Added Base URL placeholder and per-format custom path placeholder/helper text.
  - Fixed escaped slash display by rendering the default path as direct React text instead of an i18n interpolation.
- **Validation**:
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Frontend build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 17: Aether-style provider endpoint manager

- **Status**: IN_PROGRESS
- **Started**: 2026-05-11
- **Scope**:
  - Replace the simple add endpoint dialog with an Aether-style endpoint manager.
  - Add real endpoint update/delete backend APIs.
  - Support configured endpoint cards before adding more formats.
  - Add header/body rewrite rule editing with condition support.
- **Validation target**:
  - `just check`
  - `pnpm lint:frontend`
  - `pnpm build:frontend`

## Milestone 13: Provider priority management modal

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **Scope**:
  - Replace the compact priority strategy select with a modal entry point.
  - Show providers in a reorderable priority list with editable priority numbers.
  - Keep scheduling mode visible and editable in the modal footer.
  - Save provider priorities through existing provider update API and scheduling mode through existing system settings API.
- **What was done**:
  - Replaced the provider table toolbar select with a priority management modal entry and current scheduling summary.
  - Added a provider-first modal with full provider list loading, drag reorder, inline priority editing, and same-priority tier support.
  - Saved changed provider priorities through the existing provider update API and scheduling mode through the existing system settings API.
  - Added backend i18n seed strings for modal labels, validation, and success messages.
- **Validation**:
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Frontend build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 14: Provider details drawer

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **Scope**:
  - Replace the provider detail modal with a right-side Drawer.
  - Match the Settings drawer behavior with an invisible backdrop.
  - Keep the drawer mounted by rendering it as the persistent provider detail surface.
  - Preserve endpoint, key, and model binding management actions inside the drawer.
- **What was done**:
  - Replaced `ProviderBindingsPanel` Dialog with a right-side Drawer.
  - Applied Settings drawer-style invisible backdrop and paper surface styling.
  - Kept provider binding sections and add actions inside the Drawer content.
- **Validation**:
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Frontend build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 15: Provider filters toolbar alignment

- **Status**: DONE
- **Started**: 2026-05-11
- **Completed**: 2026-05-11
- **Scope**:
  - Align the provider management toolbar with Aether: provider search, status filter, API format filter, model filter, and scheduling entry in one toolbar.
  - Wire API format and model filters into the real provider list query.
  - Keep provider priority management available from the scheduling entry.
- **What was done**:
  - Added a provider-specific toolbar with search, status, API format, model, reset, scheduling, add, and refresh controls.
  - Added `api_format` and `model_id` provider list filters through frontend action types, API query DTOs, service sanitization, and storage filtering.
  - Matched Aether's provider filtering semantics: endpoint API format membership and active provider-model binding membership.
  - Updated backend admin i18n seed labels for provider search, all formats, and all models.
- **Validation**:
  - `cargo test -p storage provider_list_filters` -> exit 0
  - `just check` -> exit 0
  - `pnpm lint:frontend` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Frontend build output included the existing `Axios error: unauthorized` prerender log, but the command completed successfully.

## Milestone 17: Aether-style provider endpoint manager

- **Status**: DONE
- **Completed**: 2026-05-11
- **What was done**:
  - Replaced the simple endpoint add dialog with an Aether-style endpoint manager: configured endpoint cards plus a dashed add-format card.
  - Wired endpoint PATCH/DELETE actions to the backend update/delete routes.
  - Added typed header/body rewrite rule editing with set/drop/rename/insert/regex_replace/name_style actions.
  - Added condition editing with AND/OR groups, Current/Original sources, and Aether-aligned operators.
  - Added per-format custom path placeholders/helper text without i18n slash escaping.
  - Added real drag/drop ordering for header and body rules.
- **Validation**:
  - `just check` -> exit 0
  - `pnpm lint:frontend` -> exit 0
  - `pnpm --filter hook_frontend exec tsc --noEmit` -> exit 0
  - `pnpm build:frontend` -> exit 0
  - Frontend build output included the existing `Axios error: Something went wrong!` prerender log, but the command completed successfully.
