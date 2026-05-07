# Dual Token Auth

## Goal

Make the backend auth endpoints return both JWT access tokens and refresh tokens, support a real refresh flow, configure JWT lifetimes in existing config files, run backend tests, then wire the frontend JWT auth client to the backend response and refresh mechanism.

## Scope

- Understand current Rust backend app layout, config loading, auth handlers, storage, and tests.
- Add explicit access-token and refresh-token lifetimes to the current config model and files.
- Implement dual-token issuance and a refresh endpoint without mock or silent fallback behavior.
- Add or adjust backend tests for login and refresh behavior.
- Update the frontend JWT auth client to store/use both tokens and refresh access tokens when needed.
- Run backend unit tests and frontend validation.

## Constraints

- Keep failures explicit; do not add silent degradation or fake success paths.
- Do not introduce refresh-token persistence fallback unless the existing storage model requires it and it is explicit.
- Keep changes scoped to auth/config/frontend integration.
