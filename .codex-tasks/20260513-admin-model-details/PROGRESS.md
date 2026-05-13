# Progress Log

---

## Session Start

- **Date**: 2026-05-13 15:46 CST
- **Task name**: `20260513-admin-model-details`
- **Task dir**: `.codex-tasks/20260513-admin-model-details/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (4 milestones)
- **Environment**: Rust workspace / Next.js frontend / pnpm + just

---

## Context Recovery Block

- **Current milestone**: #4 — Fix model timestamps and base-price semantics
- **Current status**: IN_PROGRESS
- **Last completed**: #3 — Update i18n seeds
- **Current artifact**: `.codex-tasks/20260513-admin-model-details/TODO.csv`
- **Key context**: `GlobalModelResponse` already includes `usage_count`; `/api/admin/models/global/{id}/providers` returns provider bindings; `/api/admin/groups` can return billing groups up to the existing backend max list limit of 1000; display currency comes from admin system settings.
- **Known issues**:
  - Admin model detail `created_at` can render as `Invalid` because model storage still serializes timestamps with `OffsetDateTime::to_string()`, while frontend date formatting expects RFC3339-like input.
  - User requested that both user-facing and admin-facing model default pricing be labeled as base pricing, to avoid confusion with actual billing after group multipliers.
- **Next action**: Add a backend timestamp serialization test and fix, then align user/admin model pricing wording and re-run validation.

---

## Milestone 1: Inspect contracts and UI patterns

- **Status**: DONE
- **Started**: 15:46
- **Completed**: 15:58
- **What was done**:
  - Inspected admin model table/view, model catalog detail drawer, model/group/provider frontend actions and types, Rust model/group APIs, and backend admin i18n seeds.
- **Key decisions**:
  - Decision: Implement the admin detail as a new modal component under `sections/admin`.
  - Reasoning: Existing admin files are close to the 300-line file limit; the user asked for a modal rather than the existing catalog drawer.
  - Decision: Use existing model/group/provider endpoints instead of new backend endpoints.
  - Reasoning: Required data is already exposed: model rows include usage counts, group list exposes multipliers and model access, and provider bindings expose per-provider model details.
- **Problems encountered**:
  - Problem: Existing public group pricing section does not expose whether a group actually allows the selected model.
  - Resolution: Use a dedicated admin group pricing component that renders inactive/not-allowed states explicitly.
  - Retry count: 0
- **Validation**: read-only inspection commands → exit 0
- **Files changed**:
  - `.codex-tasks/20260513-admin-model-details/SPEC.md` — task scope
  - `.codex-tasks/20260513-admin-model-details/TODO.csv` — milestone status
  - `.codex-tasks/20260513-admin-model-details/PROGRESS.md` — investigation notes
- **Next step**: Milestone 2 — Implement UI behavior

## Milestone 2: Implement UI behavior

- **Status**: DONE
- **Started**: 15:58
- **Completed**: 16:13
- **What was done**:
  - Added admin model table usage count column and detail action.
  - Added `useGlobalModelProviders` for the existing model-provider endpoint.
  - Added a model detail modal with basic stats, base pricing, billing-group effective pricing, and provider bindings.
- **Key decisions**:
  - Decision: Fetch all admin groups with the existing `limit=1000`.
  - Reasoning: `1000` is the existing backend max list limit, and the modal needs full group context for actual price visibility.
  - Decision: Do not calculate prices for inactive groups or groups that do not allow the selected model.
  - Reasoning: Scheduler rejects inactive groups and groups that deny the model, so those do not have an actual model price.
- **Problems encountered**:
  - Problem: ESLint import ordering failed on newly added imports.
  - Resolution: Reordered imports according to the repository rule.
  - Retry count: 0
- **Validation**: `pnpm --filter hook_frontend lint` → exit 0
- **Files changed**:
  - `apps/hook_frontend/src/actions/models.ts` — provider binding hook
  - `apps/hook_frontend/src/sections/admin/global-model-table.tsx` — usage count and detail action
  - `apps/hook_frontend/src/sections/admin/model-management-view.tsx` — detail state and group loading
  - `apps/hook_frontend/src/sections/admin/global-model-detail-dialog.tsx` — detail modal
  - `apps/hook_frontend/src/sections/admin/global-model-billing-group-pricing.tsx` — group effective pricing
  - `apps/hook_frontend/src/sections/admin/global-model-provider-bindings.tsx` — provider binding display
- **Next step**: Milestone 3 — Update i18n seeds

## Milestone 3: Update i18n seeds

- **Status**: DONE
- **Started**: 16:10
- **Completed**: 16:13
- **What was done**:
  - Added Chinese and English admin seed translations for active group pricing, inactive group, and group-denied model states.
- **Key decisions**:
  - Decision: Keep all new copy in backend admin seed JSON.
  - Reasoning: Project i18n rules require admin copy to come from backend-controlled resources.
- **Problems encountered**:
  - Problem: None.
  - Resolution: Not applicable.
  - Retry count: 0
- **Validation**: JSON parse command → exit 0
- **Files changed**:
  - `apps/hook_backend/src/migration/defaults/i18n/admin.cn.json` — Chinese admin seed keys
  - `apps/hook_backend/src/migration/defaults/i18n/admin.en.json` — English admin seed keys
- **Next step**: Milestone 4 — Fix model timestamps and base-price semantics

## Milestone 4: Fix model timestamps and base-price semantics

- **Status**: DONE
- **Started**: 16:42
- **Completed**: 17:00
- **What is being done**:
  - Added backend tests that lock `GlobalModelResponse.created_at` / `updated_at` to RFC3339 output.
  - Replaced model timestamp serialization with the same RFC3339 formatter pattern already used by other storage entities.
  - Renamed user/admin model default pricing surfaces to base pricing so list, detail, and edit entry points no longer imply actual billed price.
- **Key decisions**:
  - Decision: Keep billing-group pricing as a separate explicit actual-pricing section instead of removing it.
  - Reasoning: The request is to stop generic model price displays from being mistaken as final billed price; group-specific multiplied prices remain useful when clearly labeled as actual pricing.
- **Validation**:
  - `cargo test -p storage model::record --lib` → exit 0
  - `pnpm lint:frontend` → exit 0
- **Files changed**:
  - `crates/storage/src/model/record.rs` — RFC3339 model timestamp formatting and regression tests
  - `apps/hook_frontend/src/sections/admin/global-model-detail-dialog.tsx` — created time now renders with `fDateTime`
  - `apps/hook_backend/src/migration/defaults/i18n/admin.cn.json` — base-pricing and actual-group-pricing wording
  - `apps/hook_backend/src/migration/defaults/i18n/admin.en.json` — base-pricing and actual-group-pricing wording
- **Next step**: Milestone 5 — Run validation

## Milestone 5: Run validation

- **Status**: DONE
- **Started**: 17:00
- **Completed**: 17:02
- **What was done**:
  - Re-ran frontend production build after the timestamp and wording changes.
  - Confirmed the targeted backend storage tests and frontend lint/build all pass on the current tree.
- **Problems encountered**:
  - Problem: Next build logs `Axios error: unauthorized` while collecting static page data.
  - Resolution: No code change in this task. The build still exits successfully, so the message is recorded as a pre-existing or adjacent integration issue rather than silently masked.
  - Retry count: 0
- **Validation**:
  - `cargo test -p storage model::record --lib` → exit 0
  - `pnpm lint:frontend` → exit 0
  - `pnpm build:frontend` → exit 0
- **Next step**: Milestone 6 — Inspect user-side detail and group visibility contracts

## Milestone 6: Inspect user-side detail and group visibility contracts

- **Status**: DONE
- **Started**: 17:15
- **Completed**: 17:18
- **What is being done**:
  - Located the user-side model catalog entry point and current detail drawer state flow.
  - Confirmed what `/api/groups/available` returns to ordinary users and whether provider bindings are still present on the wire.
  - Identified the smallest implementation that switches the user detail to a modal, keeps provider info hidden, and adds a standalone user-side price-group page.
- **Key findings so far**:
  - `ModelCatalogView` currently opens `ModelDetailDrawer` from `apps/hook_frontend/src/sections/models/model-catalog-view.tsx`.
  - `useAvailableBillingGroups()` hits `/api/groups/available`, and the backend currently returns `BillingGroupResponse`, which still includes `allowed_provider_ids`.
  - User-side token creation and model catalog use `allowed_model_ids`, but do not need provider IDs.
- **Next step**: Milestone 7 — Implement user-side modal and group page

## Milestone 7: Implement user-side modal and group page

- **Status**: DONE
- **Started**: 17:18
- **Completed**: 17:42
- **What was done**:
  - Replaced the user-side model detail drawer with a modal that shows only model information.
  - Removed billing-group content from the model modal.
  - Added a standalone `/dashboard/groups` page for user-side price groups with group name, multiplier, and allowed model names only.
  - Added default dashboard menu wiring and breadcrumb constants for the new price-group page.
- **Key decisions**:
  - Decision: Keep user model modal scoped to model-only information instead of mirroring the admin group/pricing tabs.
  - Reasoning: The latest requirement explicitly moved billing groups out of the model modal into a standalone dashboard page.
  - Decision: Name the new page/menu `价格分组`.
  - Reasoning: The user rejected `用户可见分组` as user-facing wording.
- **Validation**:
  - `pnpm --filter hook_frontend lint` → exit 0
- **Files changed**:
  - `apps/hook_frontend/src/sections/models/model-catalog-view.tsx` — modal state and removed group loading from model catalog
  - `apps/hook_frontend/src/sections/models/model-detail-dialog.tsx` — user model modal
  - `apps/hook_frontend/src/sections/models/model-detail-drawer.tsx` — removed
  - `apps/hook_frontend/src/sections/models/model-group-pricing-section.tsx` — removed
  - `apps/hook_frontend/src/sections/models/model-available-billing-groups-section.tsx` — price-group list content
  - `apps/hook_frontend/src/sections/models/billing-group-catalog-view.tsx` — standalone dashboard view
  - `apps/hook_frontend/src/app/dashboard/groups/page.tsx` — new route
  - `apps/hook_frontend/src/layouts/dashboard/dashboard-menu-values.ts` — new menu code
  - `apps/hook_backend/src/migration/defaults/menu.rs` — default menu item seed
  - `apps/hook_backend/src/migration/defaults/mod.rs` — default user menu and API binding
  - `apps/hook_backend/src/migration/defaults/i18n/admin.cn.json` — `dashboard_groups` / `价格分组`
  - `apps/hook_backend/src/migration/defaults/i18n/admin.en.json` — `dashboard_groups` / `Price Groups`
- **Next step**: Milestone 8 — Sanitize user-side available group API

## Milestone 8: Sanitize user-side available group API

- **Status**: DONE
- **Started**: 17:35
- **Completed**: 17:40
- **What was done**:
  - Sanitized `available_groups()` so user-side group responses clear `allowed_provider_ids`.
  - Added a regression test to lock the behavior.
- **Validation**:
  - `cargo test -p group --lib` → exit 0
- **Files changed**:
  - `crates/group/src/application/service.rs` — provider-binding stripping and regression test
- **Next step**: Milestone 9 — Run validation

## Milestone 9: Run validation

- **Status**: DONE
- **Started**: 17:42
- **Completed**: 17:48
- **What was done**:
  - Re-ran frontend lint.
  - Ran targeted backend test for default dashboard menu codes after adding `dashboard_groups`.
  - Re-ran frontend production build and confirmed the new route is emitted.
- **Problems encountered**:
  - Problem: `apps/hook_frontend/next-env.d.ts` was rewritten by `next build`.
  - Resolution: Reverted the generated path-only diff after validation, same as the earlier build pass.
  - Retry count: 0
- **Validation**:
  - `pnpm --filter hook_frontend lint` → exit 0
  - `cargo test -p group --lib` → exit 0
  - `cargo test -p backend default_role_menu_codes_exist` → exit 0
  - `pnpm build:frontend` → exit 0
  - Build note: one `Axios error: unauthorized` line still appears during static page data collection, but the build exits successfully.

## Milestone 10: Wire admin billing-group detail dialog

- **Status**: DONE
- **Started**: 18:02
- **Completed**: 18:08
- **What is being done**:
  - Re-opened the task after a follow-up requirement: admin billing groups also need a dedicated detail entry.
  - Verified that `BillingGroupDetailDialog` already exists and includes model plus provider visibility, but `BillingGroupManagementView` and `BillingGroupTable` do not yet wire it into the admin list workflow.
- **What was done**:
  - Added billing-group detail state to `BillingGroupManagementView` and opened the existing dialog from the admin management page.
  - Added a dedicated details action to `BillingGroupTable`, separate from edit/delete, so admins can inspect group scope without entering edit mode.
  - Kept the admin detail dialog's provider section intact, matching the confidentiality boundary requested for admin vs. user views.
- **Problems encountered**:
  - Problem: ESLint surfaced existing import-order violations in the new detail dialog file and the updated management view.
  - Resolution: Ran targeted `eslint --fix` on the affected admin files and re-ran lint.
  - Retry count: 0
- **Validation**:
  - `pnpm --filter hook_frontend lint` → exit 0
- **Next step**: Milestone 11 — Run validation for admin group detail

## Milestone 11: Run validation for admin group detail

- **Status**: DONE
- **Started**: 18:08
- **Completed**: 18:12
- **What was done**:
  - Re-ran the frontend production build after the admin billing-group detail wiring.
  - Reverted the generated `apps/hook_frontend/next-env.d.ts` diff after validation to keep the worktree focused on source changes.
- **Problems encountered**:
  - Problem: Next build still logs `Axios error: unauthorized` during static page generation.
  - Resolution: No code change in this task. The build exits successfully, so the message remains recorded as an adjacent pre-existing build-time log.
  - Retry count: 0
- **Validation**:
  - `pnpm --filter hook_frontend build` → exit 0

## Milestone 12: Inspect user model detail group-pricing tab scope

- **Status**: DONE
- **Started**: 18:20
- **Completed**: 18:23
- **What is being done**:
  - Re-opened the task after a new follow-up requirement: user model detail modal should also include a billing-group tab with actual pricing.
  - Re-checked the current user modal implementation and confirmed the earlier requirement had intentionally removed group content from the modal, so this is a new reversal rather than a missing tail from the prior step.
  - Verified that user-side available groups are already sanitized server-side and that the existing admin actual-pricing component does not depend on provider data.
- **Next step**: Milestone 13 — Implement user model detail group-pricing tab

## Milestone 13: Implement user model detail group-pricing tab

- **Status**: DONE
- **Started**: 18:23
- **Completed**: 18:31
- **What was done**:
  - Added a second tab to the user model detail modal so users can switch from basic information to group-based actual pricing.
  - Extended `useAvailableBillingGroups()` with an `enabled` flag and only fetched user price groups while the model dialog is open.
  - Kept provider information hidden and used a dedicated user-facing title `价格分组实际价格` instead of changing the admin wording.
- **Validation**:
  - `pnpm --filter hook_frontend lint` → exit 0
  - JSON parse for `admin.cn.json` / `admin.en.json` → exit 0
- **Next step**: Milestone 14 — Run validation for user model detail group-pricing tab

## Milestone 14: Run validation for user model detail group-pricing tab

- **Status**: DONE
- **Started**: 18:31
- **Completed**: 18:38
- **What was done**:
  - Re-ran the frontend build after adding the user-side price-group tab and the user-only wording key.
  - Reverted the generated `apps/hook_frontend/next-env.d.ts` diff after validation.
- **Problems encountered**:
  - Problem: Next build still logs `Axios error: unauthorized` during static page generation.
  - Resolution: No code change in this task. The build exits successfully, so the message remains tracked as a separate build-time log.
  - Retry count: 0
- **Validation**:
  - `pnpm --filter hook_frontend build` → exit 0

## Milestone 15: Inspect user price-group detail scope

- **Status**: DONE
- **Started**: 18:40
- **Completed**: 18:42
- **What was done**:
  - Checked the standalone user price-group page and confirmed it only rendered summary cards with no detail interaction.
  - Confirmed the user summary card still displayed concrete model chips even when `allowed_model_ids` was empty, which contradicted the new user-only requirement.
- **Next step**: Milestone 16 — Implement user price-group detail dialog and summary fix

## Milestone 16: Implement user price-group detail dialog and summary fix

- **Status**: DONE
- **Started**: 18:42
- **Completed**: 18:52
- **What was done**:
  - Added a user-side price-group detail dialog that shows the selected group, its bound models, each model's base pricing, and the prices after applying the group multiplier.
  - Extracted shared model actual-pricing rendering so both admin model-group pricing and user price-group details use the same price math.
  - Added a details action to each user price-group summary card.
  - Changed the user summary cards so all-model groups show only `所有模型` and do not also list concrete model names. Admin-side summaries were left unchanged.
- **Problems encountered**:
  - Problem: ESLint flagged import ordering in the new shared pricing component and the new user detail files.
  - Resolution: Ran targeted `eslint --fix` on the affected files and re-ran lint.
  - Retry count: 0
- **Validation**:
  - `pnpm --filter hook_frontend lint` → exit 0
- **Next step**: Milestone 17 — Run validation for user price-group detail dialog

## Milestone 17: Run validation for user price-group detail dialog

- **Status**: DONE
- **Started**: 18:52
- **Completed**: 18:57
- **What was done**:
  - Re-ran the frontend production build after the user price-group detail dialog and summary changes.
  - Reverted the generated `apps/hook_frontend/next-env.d.ts` diff again after validation.
- **Problems encountered**:
  - Problem: Next build still logs `Axios error: unauthorized` during static page generation.
  - Resolution: No code change in this task. The build exits successfully, so the message remains recorded as an adjacent pre-existing build-time log.
  - Retry count: 0
- **Validation**:
  - `pnpm --filter hook_frontend build` → exit 0
