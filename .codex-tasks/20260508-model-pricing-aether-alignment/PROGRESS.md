# Progress Log

## Session Start

- **Date**: 2026-05-08
- **Task name**: `20260508-model-pricing-aether-alignment`
- **Scope**: Aether-aligned tiered pricing, model.dev cache defaults, and delete success handling.

## Milestone 1: Inspect current behavior

- **Status**: DONE
- **Root causes**:
  - Hook `normalizeModel()` only maps `cost.input`, `cost.output`, and `cost.cache_read`; it never exposes cache creation or 1h cache values to the form.
  - Hook admin form stores a single flat price row in `GlobalModelForm`, so users cannot add Aether-style `default_tiered_pricing.tiers`.
  - Hook backend and TypeScript types already support `cache_ttl_pricing` and multiple tiers, so this is a frontend editing/submission gap rather than a schema gap.
  - `deleteGlobalModel()` calls `requestData<void>()`; DELETE returns `data: null`/void-like JSON, and `requireApiData` correctly rejects missing data for read APIs, causing a false failure toast after the backend deletion succeeds.

## Milestone 2: Implementation

- **Status**: DONE
- **Plan**:
  - Add a focused React tiered pricing editor mirroring Aether's tier threshold and cache auto-calculation rules.
  - Store `tiered_pricing` in the admin form and keep existing single-price field names only where still needed for table/detail helpers.
  - Treat DELETE as a void success path in `actions/models.ts`, then invalidate model list SWR keys.
- **What changed**:
  - Added shared Aether pricing helpers for cache creation/read/1h cache calculation.
  - Added an admin `TieredPricingEditor` that supports multiple tiers, tier limits, deleting tiers, and an unlimited final tier.
  - Updated model.dev normalization and form fill so selected models include cache creation, cache read, and 1h cache values.
  - Updated create/update payload construction to submit full `default_tiered_pricing`.
  - Updated DELETE handling to require only the API success flag, not non-null `data`.

## Milestone 3: Validation

- **Status**: DONE
- **Commands**:
  - `pnpm --filter hook_frontend exec eslint src/actions/models.ts src/types/model.ts src/utils/model-pricing.ts src/sections/admin/model-management-utils.ts src/sections/admin/global-model-form-dialog.tsx src/sections/admin/tiered-pricing-editor.tsx src/sections/admin/tiered-pricing-utils.ts` -> passed.
  - `pnpm --filter hook_frontend exec tsc --noEmit --pretty false` -> passed.
  - `pnpm lint:frontend` -> passed.
  - `pnpm build:frontend` -> passed.
- **Migration note**:
  - No new migration is needed for this fix. Existing Hook backend types and storage already support `default_tiered_pricing.tiers` and `cache_ttl_pricing`; the missing behavior was in the admin frontend.
