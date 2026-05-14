# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Request history must retain provider/key display fields after the provider or provider key is deleted.
- Request history must retain user/token display fields after the token is deleted.
- Deleting a user through the existing API path must also delete that user's API tokens.
- Query and detail APIs must read persisted snapshots for display/search instead of depending on live association rows.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- No frontend redesign.
- No compatibility fake data or mock success path.
- No unrelated module refactor.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Rust workspace with SeaORM entities and baseline migration files.
- Backend tests must run with a 60 second timeout.
- Keep failures explicit; do not hide missing data with silent fake values.
- Preserve unrelated user work and avoid destructive git operations.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024 workspace, Next.js frontend present but not in scope
- **Package manager**: Cargo, pnpm
- **Test framework**: Cargo tests
- **Build command**: `just check`
- **Existing test count**: determined by targeted Cargo test runs

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — no external service required
- [ ] Breaking changes to existing code — impact assessed?
- [x] Large file generation — not applicable
- [x] Long-running tests — timeout configured

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Request record and candidate schema/entity snapshot fields.
- Proxy audit write path that captures provider/key/user/token/model snapshots at request time.
- Request record list/detail/search reads backed by snapshot fields.
- User deletion path that deletes user API tokens and invalidates proxy auth cache.
- Targeted tests for snapshot retention and user token cleanup.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Provider/key/user/token names remain queryable from request history after associated rows are deleted.
- [ ] User deletion removes API tokens rather than leaving usable token rows behind.
- [ ] Relevant targeted Cargo tests pass with 60 second timeout.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_records && perl -e 'alarm shift; exec @ARGV' 60 cargo test -p user
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Create a request record with provider/key/user/token snapshots.
2. Remove associated provider/key/token rows.
3. Query request record list/detail and confirm the persisted display names remain.
