# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Fix the request-records page refresh Runtime Error.
- Make the request-records toolbar refresh label show a translated value.

## Non-Goals

- Do not change request-records backend behavior because the provided curl returns a valid success envelope.
- Do not add fallback translation resources or silent degradation.

## Constraints

- Admin UI copy must come from backend i18n resources.
- Do not pass DOM events into SWR `mutate`.
- Keep changes scoped to the request-records admin UI unless validation exposes a direct dependency.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js frontend, Rust backend
- **Package manager**: `pnpm`
- **Test framework**: frontend lint/build checks
- **Build command**: `pnpm build:frontend`
- **Existing test count**: JavaScript test runner not configured

## Risk Assessment

- [x] External dependencies (APIs, services) — request-records API curl returns `{"success":true,"message":"","data":{"records":[],"total":0}}`
- [x] Breaking changes to existing code — impact scoped to refresh callback invocation
- [x] Large file generation — none
- [x] Long-running tests — frontend lint only for this fix

## Deliverables

- Updated request-records refresh handlers.
- Updated request-records toolbar refresh label key.

## Done-When

- [ ] Clicking refresh cannot write the click event into the SWR cache.
- [ ] Request-records toolbar refresh button uses an available admin translation key.
- [ ] Frontend lint passes or reports an unrelated existing failure.

## Final Validation Command

```bash
pnpm --filter hook_frontend lint
```
