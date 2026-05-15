# Progress

## Current state
- Added `crates/req`, wired it into the workspace, and migrated suitable JSON/HTTP consumers.
- `cargo check -p backend -p model -p provider -p req` passes.
- `cargo check --workspace` passes.
- `cargo test -p req` passes.
- `just test` passes.

## Notes
- The main design choice is whether to mirror the full `gem_client` surface or keep only the shared plumbing that the current workspace actually uses.
- Current scope keeps reusable HTTP plumbing and avoids moving protocol-specific request shaping out of owner crates.
- LLM proxy streaming paths still use native `reqwest` types because response streaming, per-attempt timeout, and error classification depend on those concrete types.
