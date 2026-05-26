# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Rework Hook streaming proxy termination handling so incomplete provider streams are recorded as explicit failures.
- Add a preflight stage before opening downstream streaming bodies.
- Route stream timeout/read/incomplete failures into the existing provider cooldown mechanism.
- Keep failures visible in request records and logs without compatibility success paths.

## Non-Goals

- Do not add a new database schema or separate provider health system.
- Do not import Aether tunnel or remote execution runtime.
- Do not add fallback/simulation paths that hide provider failures.

## Constraints

- Follow repository AGENTS.md rules: Chinese user-facing updates, Debug-First, no silent fallbacks.
- Use existing `request_records`, `request_candidates`, and `provider_cooldowns` fields.
- Reference `/Users/bubu/Downloads/Aether-main` for uncertain stream execution details.
- Keep Rust unit tests under the repository 60-second timeout rule.

## Environment

- **Project root**: `/Users/bubu/.codex/worktrees/8ce0/Hook`
- **Language/runtime**: Rust workspace + pnpm frontend monorepo
- **Package manager**: Cargo, pnpm
- **Test framework**: Rust `cargo test`
- **Build command**: `just build`
- **Existing test count**: not enumerated at task start

## Risk Assessment

- [x] External dependencies (APIs, services) — online evidence collected from 50.16.57.26 before implementation.
- [x] Breaking changes to existing code — intentional: incomplete provider stream no longer records success.
- [x] Large file generation — no large generated files expected.
- [x] Long-running tests — validation commands run with `timeout 60s` where applicable.

## Evidence

- Online 24h request records showed `success/done` 4207, `failed/upstream_timeout` 182, `success/upstream_eof_without_completion` 101, `cancelled/client_gone` 13.
- Abnormal stream logs included `stream idle timeout`, `Network error: error decoding response body`, and `upstream_eof_without_completion`.
- Aether references:
  - `/Users/bubu/Downloads/Aether-main/apps/aether-gateway/src/execution_runtime/stream/execution.rs`
  - `/Users/bubu/Downloads/Aether-main/apps/aether-gateway/src/execution_runtime/stream/error.rs`
  - `/Users/bubu/Downloads/Aether-main/apps/aether-gateway/src/handlers/admin/provider/pool/runtime/writes.rs`

## Deliverables

- Stream terminal summary type and centralized terminal record construction.
- Preflight failure handling before downstream stream creation.
- Incomplete stream EOF recorded as failed with `upstream_incomplete_stream`.
- Stream failure cooldown recording through existing provider cooldown storage.
- Focused Rust tests for stream terminal classification and cooldown behavior where feasible.

## Done-When

- [ ] Incomplete provider streams are no longer recorded as success.
- [ ] Stream first-byte and idle timeouts are visibly recorded as failures.
- [ ] Client disconnect remains `cancelled/client_disconnected` and does not become provider failure.
- [ ] Stream provider failures can trigger existing provider cooldown.
- [ ] Targeted Rust validations run and results are logged.

## Final Validation Command

```bash
timeout 60s cargo test -p hook_backend stream_transport && timeout 60s cargo test -p storage provider_request
```
