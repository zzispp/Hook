# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Add a request-recording section to the admin system settings page.
- Persist settings that control request/response detail storage: detail level, max request body size, max response body size, sensitive headers, and switches for request headers, request body, and response body.
- Follow Hook repository patterns for backend settings, frontend admin UI, and backend-seeded admin i18n.
- Use Aether as behavior/UI reference without copying unrelated architecture.

## Non-Goals

- Do not implement request-record middleware behavior in this task.
- Do not add frontend locale JSON files.
- Do not add fallback/mock success paths.

## Constraints

- Admin UI copy must come from backend i18n seed resources.
- Backend validation must expose invalid input as explicit errors.
- Keep changes scoped to the setting section and related types/translations.
- Backend unit tests must run with a 60-second timeout.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace, Next.js/TypeScript frontend
- **Package manager**: pnpm
- **Test framework**: Rust cargo tests/checks, frontend ESLint/Next build
- **Build command**: `just check`, `pnpm lint:frontend`
- **Existing test count**: not pre-counted

## Risk Assessment

- [x] External dependencies (APIs, services) — local source only.
- [x] Breaking changes to existing code — impact to system settings API/types assessed before editing.
- [x] Large file generation — no large generated artifacts expected.
- [x] Long-running tests — backend commands run with timeout where applicable.

## Deliverables

- Backend setting key defaults and validation for request-record configuration.
- Frontend settings API/types and admin settings UI section.
- Backend admin i18n seed entries for Chinese and English UI copy.

## Done-When

- [ ] Settings API can read/write the new keys with explicit validation.
- [ ] Admin settings page shows and saves the request-record controls.
- [ ] Fresh development i18n seed contains required admin copy.
- [ ] Validation commands pass or any failure is reported with evidence.

## Final Validation Command

```bash
just check && pnpm lint:frontend
```
