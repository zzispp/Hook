# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Make scheduled task execution safe under multi-instance blue/green backend deployments.
- Ensure every registered scheduled task is claimed globally from the database before execution.
- Keep future scheduled task registrations covered by the shared scheduler runtime and storage layer.

## Non-Goals

- Do not change individual task business behavior.
- Do not add mock success paths, silent fallbacks, or per-task special cases.
- Do not change production deployment topology.

## Constraints

- Preserve existing Rust workspace patterns and SeaORM storage style.
- Keep scheduler failures visible through task run records and logs.
- Use database state as the cross-instance source of truth for due time and execution leases.
- Respect the repository limits from `AGENTS.md` for function size, nesting, and explicit edge cases.

## Environment

- **Project root**: `/Users/bubu/.codex/worktrees/df4a/Hook`
- **Language/runtime**: Rust 2024 workspace, Next.js frontend unaffected
- **Package manager**: pnpm for frontend, Cargo/just for Rust
- **Test framework**: Rust unit/integration tests
- **Build command**: `just check`
- **Existing test count**: not precomputed

## Risk Assessment

- [x] External dependencies (APIs, services) — not needed for implementation.
- [x] Breaking changes to existing code — scheduler storage/runtime impact assessed.
- [x] Large file generation — not expected.
- [x] Long-running tests — use repository `just test` wrapper or targeted Cargo tests.

## Deliverables

- Scheduler table migration adds global due-time claim state.
- Storage repository exposes atomic due task claim and lease release/update operations.
- Scheduler runtime uses the shared claim path for all tasks instead of local-only due decisions.
- Tests cover multi-instance claim behavior and future tasks through the shared runtime path.

## Done-When

- [ ] A due task can be claimed by only one scheduler instance per due window.
- [ ] A second instance skips without creating a successful task run when the due row is already claimed or not due.
- [ ] A stale lease can be reclaimed after expiration.
- [ ] Existing scheduled task APIs continue to list and update tasks.
- [ ] Targeted Rust tests and workspace check pass.

## Final Validation Command

```bash
timeout 60s cargo test -p scheduler -p storage && just check
```
