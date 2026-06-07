# Progress

- Created task context after resuming `codex/fix-nonstream-terminal-guard` in `/Users/bubu/.codex/worktrees/probe-cancel/Hook`.
- Initial diff already moves `AttemptCancelGuard::disarm()` to run after awaited terminal writers but before propagating their error in non-stream full response, stream upstream status failure, and image response paths.
- Root cause: after upstream response start, non-stream terminal writers can return an error after recording the real terminal failure; if `AttemptCancelGuard` is still armed when that error propagates, `Drop` records `cancelled / 499` over the real terminal state.
- Added tests for the guard's awaiting-terminal phase: armed state yields a cancellation phase; disarmed state yields no cancellation phase.
- Validation passed: `cargo test -p hook_backend attempt_log`, `cargo test -p hook_backend executor`, `cargo test -p hook_backend transport_read`, `cargo test -p hook_backend llm_proxy::proxy`, and `cargo check`.
