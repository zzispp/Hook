# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Make token-copy actions work when the frontend is served from an HTTP intranet origin.
- Make adding an API endpoint work when `crypto.randomUUID` is unavailable.
- Load the models.dev catalog directly from the browser so backend egress is not required.
- Remove the obsolete backend models.dev proxy API and its dedicated infrastructure.
- Preserve the existing HTTPS behavior and user-facing success/error semantics.

## Non-Goals

- Changing authentication, deployment TLS, models.dev data semantics, or unrelated admin forms.
- Adding silent mock behavior or suppressing copy failures.

## Constraints

- Use explicit browser-capability adapters and expose real failures.
- Keep frontend files under 300 lines and functions under 50 lines.
- No new runtime dependency unless the existing platform cannot provide the required primitive.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / React 19 / Next.js 16 / Node.js >=22.12
- **Package manager**: pnpm 10.33.4
- **Test framework**: none configured; use deterministic Node/TypeScript harness plus lint/build
- **Build command**: `pnpm build:frontend`
- **Existing test count**: no frontend test suite configured

## Risk Assessment

- [x] External dependencies: the model picker requires browser access to `https://models.dev`.
- [x] Breaking changes: `/api/admin/models/external` is intentionally removed.
- [x] Large file generation: none.
- [x] Long-running tests: frontend build/lint only; no backend test timeout needed.

## Deliverables

- Regression coverage for HTTP-compatible UUID generation and clipboard copying.
- Shared browser utilities used by affected frontend call sites.
- Browser-side models.dev client with explicit HTTP and payload validation.
- Removal of the backend models.dev route, client, port, dependency injection, and RBAC seed.
- Successful frontend type/build/lint and backend check validation.

## Done-When

- [x] A deterministic failure loop reproduces both missing-browser-API symptoms before the fix.
- [x] UUID creation succeeds when `crypto.randomUUID` is absent but `getRandomValues` exists.
- [x] Copy succeeds when `navigator.clipboard` is unavailable and reports a concrete error when no copy mechanism works.
- [x] All affected call sites use the shared utilities.
- [x] The model picker fetches `https://models.dev/api.json` without Hook authorization headers.
- [x] Invalid models.dev status codes and payload structures surface explicit errors.
- [x] Official provider metadata matches the removed backend implementation.
- [x] No frontend or backend reference to `/api/admin/models/external` remains.
- [x] Frontend lint/build and backend checks pass.

## Final Validation Command

```bash
node .codex-tasks/20260807-http-browser-api-fix/regression.mjs && pnpm lint:frontend && pnpm build:frontend && cargo check -p hook_backend
```

## Demo Flow

1. Open the application through an HTTP intranet address.
2. Copy a token from token management and confirm the expected copied state.
3. Open system settings, add an API endpoint, and confirm the editor row appears without a page crash.
