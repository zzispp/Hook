# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Add a login/register captcha feature aligned with the existing Rust backend, Next.js frontend, system settings, and i18n conventions.
- Implement the captcha server behavior in the Rust backend after inspecting `https://github.com/tiagozip/cap`.
- Reuse or port the Cap frontend behavior into the existing Next.js stack where practical.
- Add a base system setting switch that enables or disables captcha for login/register.

## Non-Goals

- Do not add silent fallback captcha success paths or mock success behavior.
- Do not introduce unrelated auth, risk-control, or UI redesign changes.
- Do not restore frontend admin locale JSON files.

## Constraints

- Follow repository `AGENTS.md` and project conventions.
- Backend failures must surface explicitly.
- Keep business logic injectable and testable.
- Admin UI copy must use backend-controlled `admin` namespace resources.
- Backend unit tests must use a 60-second timeout.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + pnpm/Next.js apps
- **Package manager**: `pnpm`
- **Test framework**: Rust `cargo test`; frontend lint/build checks
- **Build command**: `just check`, `pnpm lint:frontend`, targeted builds as needed
- **Existing test count**: To be inspected

## Risk Assessment

- [ ] External dependency behavior from `cap` repo inspected.
- [ ] Existing auth API and frontend login/register flows identified.
- [ ] System settings read/write semantics identified before adding the switch.
- [ ] i18n seed path and frontend `t()` usage validated.

## Deliverables

- Backend captcha challenge and verification endpoints.
- Backend auth login/register validation respecting the setting switch.
- Frontend captcha widget in login/register when enabled.
- Base settings switch in system settings.
- Backend i18n seed keys for new admin UI copy.
- Focused validation commands and test coverage where feasible.

## Done-When

- [ ] Captcha disabled: existing login/register behavior remains unchanged.
- [ ] Captcha enabled: login/register requires a valid captcha token.
- [ ] Captcha endpoints verify real challenge solutions and expose failures clearly.
- [ ] System base setting switch persists and controls the feature.
- [ ] Admin UI text uses backend i18n resources.
- [ ] Validation commands complete or failures are reported with concrete evidence.

## Final Validation Command

```bash
timeout 60 just test && pnpm lint:frontend
```

