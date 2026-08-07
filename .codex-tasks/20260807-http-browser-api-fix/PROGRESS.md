# Progress Log

## Session Start

- **Date**: 2026-08-07
- **Task name**: `20260807-http-browser-api-fix`
- **Task dir**: `.codex-tasks/20260807-http-browser-api-fix/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv` (6 milestones)
- **Environment**: TypeScript / Next.js 16 / no configured frontend test runner

## Context Recovery Block

- **Current milestone**: Complete
- **Current status**: DONE
- **Last completed**: #6 — Run full validation and cleanup
- **Current artifact**: final validated worktree
- **Key context**: Browser compatibility fixes pass. The deployed backend times out after 15 seconds fetching models.dev, while this machine reaches it in about two seconds and the response permits any CORS origin. The user requested browser-side fetching and complete backend proxy removal.
- **Known issues**: Playwright is not installed; the feedback loop must use existing tooling.
- **Next action**: None.

## Milestone 1: Build and run deterministic failure reproductions

- **Status**: DONE
- **Started**: 17:35
- **Completed**: 17:36
- **What was done**:
  - Added `repro-current.mjs` to invoke the current endpoint updater and third-party copy Hook under missing HTTP browser capabilities.
  - Confirmed the deployed external-model request separately: the app responds after about 15 seconds with `external service error: Timeout error`.
- **Key decisions**:
  - Use the actual source updater and Hook instead of a static source scan.
  - Keep external-model networking as a diagnosed deployment concern; do not return mock catalog data.
- **Problems encountered**:
  - Playwright is not installed in the repository; browser verification uses the connected browser and the code harness uses Node 24.
- **Validation**: `node .codex-tasks/20260807-http-browser-api-fix/repro-current.mjs` → exit 1; both expected failures captured.
- **Files changed**:
  - `.codex-tasks/20260807-http-browser-api-fix/repro-current.mjs` — deterministic red reproduction.
- **Next step**: Run the reproduction and record its exact failure output.

## Milestone 2: Trace affected call sites and test ranked hypotheses

- **Status**: DONE
- **Started**: 17:36
- **Completed**: 17:48
- **What was done**:
  - Identified two direct `crypto.randomUUID()` call sites and eleven clipboard call sites, including the token-management flow.
  - Ranked secure-context capability absence, ignored Hook result, deployment egress timeout, and extension interception as separate hypotheses.
- **Key decisions**:
  - Introduce one browser capability module and migrate all existing app copy actions so behavior is consistent.
  - Leave the external catalog's real network error visible; offline mock data would conceal the deployment defect.
- **Problems encountered**:
  - The connected browser has wallet/translation extensions that emit unrelated injection errors; page errors will be filtered by origin.
- **Validation**: `rg -n "crypto\\.randomUUID|navigator\\.clipboard" apps/hook_frontend/src` → no direct UUID use; only the intentional DevTools script literal contains `navigator.clipboard`.
- **Files changed**:
  - `.codex-tasks/20260807-http-browser-api-fix/TODO.csv` — milestone state.
- **Next step**: Milestone 3 — complete implementation validation.

## Milestone 3: Implement shared browser utilities and regression coverage

- **Status**: DONE
- **Started**: 17:39
- **Completed**: 18:21
- **What was done**:
  - Added `createUuid()` using native UUID generation or cryptographic random bytes.
  - Added `copyText()` using the native Clipboard API and an explicit legacy command path for HTTP origins.
  - Migrated token, endpoint, model, request, provider, card-code, affiliate, and landing-page copy actions.
  - Ensured success notifications occur only after copying completes.
- **Key decisions**:
  - Attempt the legacy command after a native Clipboard rejection so extension-intercepted failures can still copy under the user gesture.
  - Throw a combined concrete error if neither mechanism works.
- **Problems encountered**:
  - ESLint forbids bitwise operators; UUID version/variant bits use equivalent modulo arithmetic.
- **Validation**: `node .codex-tasks/20260807-http-browser-api-fix/regression.mjs` → `REGRESSION_GREEN`; `pnpm lint:frontend` → exit 0; `tsc --noEmit` → exit 0.
- **Files changed**:
  - `apps/hook_frontend/src/utils/browser-compat.ts` — browser capability strategies.
  - Frontend UUID and copy call sites — migrated to shared strategies.
- **Next step**: Milestone 4 — build a red-capable models.dev frontend regression.

## Scope Extension: Browser-side models.dev catalog

- **Status**: DONE
- **Started**: 18:21
- **Evidence**:
  - Deployed `GET /api/admin/models/external` returns `external service error: Timeout error` after about 15 seconds.
  - Direct `GET https://models.dev/api.json` returns HTTP 200 in about two seconds from the current machine.
  - The response includes `Access-Control-Allow-Origin: *`, so an HTTP intranet page may fetch the HTTPS resource directly.
- **Decision**:
  - Use native browser `fetch`, not the Hook axios instance, so the Hook bearer token cannot be attached to the external origin.
  - Validate response status and structural boundaries before normalizing data.
  - Preserve the official-provider metadata currently added by the backend.
  - Remove the backend route and all dedicated application/infrastructure code per the user's updated requirement.

## Milestone 4: Build models.dev frontend regression

- **Status**: DONE
- **Started**: 18:24
- **Completed**: 18:26
- **What was done**:
  - Added request-boundary cases for target URL and request headers.
  - Added explicit non-2xx, invalid top-level, invalid provider, invalid models map, and invalid model cases.
  - Added official and non-official provider assertions.
- **Validation**: `node .codex-tasks/20260807-http-browser-api-fix/regression.mjs` -> expected exit 1 because `src/utils/models-dev.ts` does not exist before implementation.
- **Next step**: Milestone 5 — implement the browser client and remove the backend API.

## Milestone 5: Move models.dev to the browser and remove the backend API

- **Status**: DONE
- **Started**: 18:26
- **Completed**: 18:49
- **What was done**:
  - Added `src/utils/models-dev.ts` with an injected native-fetch boundary, explicit HTTP/shape validation, and official-provider metadata.
  - Changed the model action to consume the browser client and removed the old axios endpoint.
  - Removed the Rust external-catalog port, client, handler, route, error variant, HTTP dependency, and startup injection.
  - Removed the default RBAC API and menu binding and added an idempotent migration to delete stale deployed metadata.
- **Validation**:
  - Regression, TypeScript, ESLint, and rustfmt checks pass.
  - `cargo check -p hook_backend` passes.
  - Live models.dev data parsed successfully: 180 providers and 6218 models.
  - `cargo test -p hook_backend migration::defaults::tests -- --nocapture` passes all 6 selected tests under a 180-second timeout.
- **Next step**: Milestone 6 — production builds and final cleanup.

## Milestone 6: Run full validation and cleanup

- **Status**: DONE
- **Started**: 18:49
- **Completed**: 19:13
- **What was done**:
  - Built both the standard Next.js application and embedded static export.
  - Confirmed production output contains the direct models.dev URL and omits the retired backend path.
  - Served the updated frontend over HTTP and verified the external request in a real Chrome page context.
  - Removed the temporary CORS probe and reduced Cargo.lock to the single intended dependency change.
  - Audited all modified copy/UUID call sites and checked for debug instrumentation and stale route references.
- **Problems encountered**:
  - Starting Next dev after a static export reused a corrupt generated `.next/dev/types/validator.ts`; moving that generated directory aside and restarting produced a valid file, after which TypeScript passed while the server remained active.
- **Validation**:
  - `node .codex-tasks/20260807-http-browser-api-fix/regression.mjs` -> `REGRESSION_GREEN`.
  - `pnpm --filter hook_frontend exec tsc --noEmit --pretty false` -> exit 0.
  - `pnpm lint:frontend` -> exit 0.
  - `pnpm build:frontend` -> exit 0.
  - `pnpm build:frontend:embedded` -> exit 0.
  - `cargo fmt --all -- --check` -> exit 0.
  - `cargo check -p hook_backend` -> exit 0.
  - `timeout 180 cargo test -p model` -> exit 0.
  - `timeout 180 cargo test -p hook_backend migration::defaults::tests -- --nocapture` -> 6 passed.
  - Browser CORS probe from `http://127.0.0.1:8082` -> HTTP 200, 180 providers, 6218 models in about 2.6 seconds.
