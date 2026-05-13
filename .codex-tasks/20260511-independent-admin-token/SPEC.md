# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Fix admin creation of independent API tokens with `user_id: null`.
- Preserve strict validation for user-owned API tokens.
- Keep failures explicit; do not create fake users or silently rewrite ownership.

## Non-Goals

- Do not change authentication token validation.
- Do not add compatibility fallback behavior.
- Do not change frontend payload semantics unless backend evidence requires it.

## Constraints

- The configured administrator is a system user supplied by configuration and is not stored in `users`.
- Existing `api_tokens.user_id` is non-null with a user foreign key, which conflicts with independent token semantics.
- Rust backend tests should run with a 60 second timeout where feasible.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust backend, TypeScript frontend
- **Package manager**: `pnpm`
- **Test framework**: `cargo test`
- **Build command**: `just build` / `pnpm build:frontend`
- **Existing test count**: inspect through cargo test output

## Risk Assessment

- [x] External dependencies — local backend on `localhost:5555` returned the reported error.
- [x] Breaking changes — storage schema and domain type may need nullable user owner.
- [x] Large file generation — none.
- [x] Long-running tests — use timeout for backend tests.

## Deliverables

- Backend service no longer requires a users table row for independent admin tokens.
- Storage/domain schema supports `user_id: null` for independent tokens.
- Tests cover independent admin token creation by a system actor.

## Done-When

- [ ] User tokens still require a concrete `user_id`.
- [ ] Independent tokens created by admin can have no `user_id`.
- [ ] Relevant Rust tests pass.

## Final Validation Command

```bash
timeout 60 cargo test -p api_token
```
