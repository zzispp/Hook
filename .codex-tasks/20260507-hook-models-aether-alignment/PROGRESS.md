# Progress Log

---

## Session Start

- **Date**: 2026-05-07 23:10
- **Task name**: `20260507-hook-models-aether-alignment`
- **Task dir**: `.codex-tasks/20260507-hook-models-aether-alignment/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust workspace / Next.js frontend / Rust tests and Next build-lint

---

## Context Recovery Block

- **Current milestone**: #2 — Map Aether model feature
- **Current status**: IN_PROGRESS
- **Last completed**: #1 — Map Hook architecture
- **Current artifact**: `TODO.csv`
- **Key context**: Hook backend uses Axum composition in `apps/hook_backend`, domain crates under `crates/*`, Toasty-backed storage in `crates/storage`, and API/domain DTOs in `crates/types`. Frontend uses Next.js/MUI with SWR actions, route constants, and dashboard sections.
- **Known issues**: Aether provider/key/routing features depend on provider tables Hook does not have yet.
- **Next action**: Finish Aether model schema/API/frontend mapping and record the aligned Hook boundary.

## Milestone 1: Map Hook architecture

- **Status**: DONE
- **Started**: 23:10
- **Completed**: 23:17
- **What was done**:
  - Read Hook root and backend instructions.
  - Mapped backend composition root, shared crates, storage schema registration, RBAC route/auth integration, and frontend admin patterns.
- **Key decisions**:
  - Decision: Add model functionality as a shared crate plus storage/types modules, then wire it from `apps/hook_backend`.
  - Reasoning: Existing Hook keeps business behavior out of the binary app and exposes feature routes from domain crates.
- **Problems encountered**:
  - Problem: None.
  - Resolution: Not applicable.
  - Retry count: 0
- **Validation**: `test -f .codex-tasks/20260507-hook-models-aether-alignment/PROGRESS.md` → exit 0
- **Files changed**:
  - `.codex-tasks/20260507-hook-models-aether-alignment/PROGRESS.md` — recorded architecture findings.
- **Next step**: Milestone 2 — Map Aether model feature

## Milestone 2: Map Aether model feature

- **Status**: DONE
- **Started**: 23:17
- **Completed**: 23:24
- **Aether model feature map**:
  - Backend router prefix is `/api/admin/models`, with child routers for `/global`, `/catalog`, `/external`, and `/global/{id}/routing`.
  - Core persistence is `global_models` plus `models`.
  - `global_models` columns: `id`, `name`, `display_name`, `default_price_per_request`, `default_tiered_pricing`, `supported_capabilities`, `config`, `is_active`, `usage_count`, `created_at`, `updated_at`.
  - `models` columns: `id`, `provider_id`, `global_model_id`, `provider_model_name`, `provider_model_mappings`, `price_per_request`, `tiered_pricing`, provider capability booleans, `is_active`, `is_available`, `config`, `created_at`, `updated_at`.
  - GlobalModel APIs expose Aether-shaped query params and responses: `skip`, `limit`, `is_active`, `search`; list response `{ models, total }`; detail response adds stats and `price_range`.
  - model.dev proxy fetches `https://models.dev/api.json`, marks official providers, and exposes it through `/api/admin/models/external`.
  - Current model.dev live shape is provider map. Model modalities now live under `modalities.input` and `modalities.output`; legacy `input`/`output` fields should still be tolerated at the frontend normalization boundary.
  - Aether frontend `/admin/models` is a management table plus model.dev import picker. Selecting a model fills `name`, `display_name`, `default_tiered_pricing`, `supported_capabilities`, and `config`.
- **Key decisions**:
  - Decision: Implement the real GlobalModel closed loop first; do not fake Provider, API Key, or routing preview behavior in Hook.
  - Reasoning: Hook does not currently have Aether's provider/key/routing domain. Fake provider data would violate Debug-First and drift business logic.
  - Decision: Keep path and DTO shape aligned with Aether for GlobalModel, catalog, provider list, and model.dev proxy.
  - Reasoning: These are useful independently and provide the same management workflow boundary.
- **Validation**: `rg -n "Aether model feature map" .codex-tasks/20260507-hook-models-aether-alignment/PROGRESS.md` → exit 0
- **Files changed**:
  - `.codex-tasks/20260507-hook-models-aether-alignment/PROGRESS.md` — recorded Aether feature map.
- **Next step**: Milestone 3 — Design Hook 1:1 alignment

## Milestone 3: Design Hook 1:1 alignment

- **Status**: DONE
- **Started**: 23:24
- **Completed**: 23:24
- **Hook alignment plan**:
  - Add `crates/model` as the model business/API crate.
  - Add `types::model` DTOs mirroring Aether response/request fields.
  - Add `storage::model` with `GlobalModelRecord` and `ModelRecord`, registered in `Database::models!`.
  - Use `rust_decimal::Decimal` for Aether `Numeric(20,8)` money fields and Toasty `#[column(type = numeric(20,8))]`.
  - Store Aether JSON/JSONB fields through Toasty `#[serialize(json)]`; the generated DB type is not literal JSONB, but the API shape and Rust field shape remain aligned.
  - Add routes under `/api/admin/models`: `/global`, `/global/{id}`, `/global/batch-delete`, `/global/{id}/providers`, `/catalog`, `/external`.
  - Add RBAC API permissions and menu item for `/dashboard/admin/models`; add CORS PATCH because Aether uses PATCH for GlobalModel updates.
  - Frontend adds `src/types/model.ts`, `src/actions/models.ts`, `src/sections/admin/model-management-view.tsx`, and `src/app/dashboard/admin/models/page.tsx`.
  - Frontend model.dev picker fetches via backend, normalizes provider/model data, and fills the GlobalModel create/edit form using Aether-compatible fields.
- **Known non-1:1 limits**:
  - Hook cannot enforce `models.provider_id -> providers.id` because no `providers` table exists. The field will exist and be indexed, but FK behavior is intentionally not invented.
  - Hook will not expose Aether routing preview as successful fake data. If/when Provider/Key routing exists, that endpoint can be added with real data.
- **Validation**: `rg -n "Hook alignment plan" .codex-tasks/20260507-hook-models-aether-alignment/PROGRESS.md` → exit 0
- **Files changed**:
  - `.codex-tasks/20260507-hook-models-aether-alignment/PROGRESS.md` — recorded Hook alignment plan.
- **Next step**: Milestone 4 — Implement backend model module

## Milestone 4: Implement backend model module

- **Status**: DONE
- **Started**: 23:24
- **Completed**: 2026-05-08 01:03
- **What was done**:
  - Added `crates/model`, `types::model`, and `storage::model`.
  - Added `global_models` and `models` Toasty records with Aether-aligned fields.
  - Exposed `/api/admin/models/global`, `/catalog`, and `/external`.
  - Wired backend startup, RBAC permissions, admin menu, and PATCH CORS support.
- **Key decisions**:
  - Decision: Keep `model.dev` as a real proxy to `https://models.dev/api.json` with explicit HTTP/shape errors.
  - Reasoning: This avoids fake imports or silent fallback paths and keeps upstream changes visible.
- **Validation**: `cargo check` → exit 0
- **Files changed**:
  - `Cargo.toml`, `apps/hook_backend/*`, `crates/model/*`, `crates/storage/src/model/*`, `crates/types/src/model/*`.
- **Next step**: Milestone 5 — Implement frontend admin model page

## Milestone 5: Implement frontend admin model page

- **Status**: DONE
- **Completed**: 2026-05-08 01:16
- **What was done**:
  - Added Aether-aligned model TypeScript types and API actions.
  - Added `/dashboard/admin/models` route, admin navigation translation, endpoints, and menu icon support.
  - Added model management table, create/edit dialog, and model.dev picker with provider/model flattening.
  - Model.dev import fills Aether-compatible `name`, `display_name`, tiered pricing, supported capabilities, and config.
- **Validation**:
  - `pnpm lint:frontend` → exit 0
  - `pnpm build:frontend` → exit 0

## Milestone 6: Run final validation

- **Status**: DONE
- **Completed**: 2026-05-08 01:16
- **Validation**:
  - `cargo check` → exit 0
  - `pnpm lint:frontend` → exit 0
  - `pnpm build:frontend` → exit 0
- **Notes**:
  - `just` is not available in this environment, so Rust validation used direct `cargo check`.
  - `next build` rewrites `apps/hook_frontend/next-env.d.ts`; the generated-file diff was restored after successful build.
