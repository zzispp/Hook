# req crate expansion

## Goal
Add reusable HTTP response and error helpers to `crates/req`, then migrate generic request handling in `provider` and `llm_proxy` to reuse them.

## Scope
- Add generic response-body and header helpers to `crates/req`.
- Add generic `reqwest::Error` classifiers to `crates/req`.
- Reuse those helpers in `provider` and `llm_proxy` where the code is protocol-neutral.
- Keep request conversion, audit logging, candidate routing, and websocket orchestration inside `llm_proxy`.

## Validation
- Workspace builds successfully.
- `cargo test -p req` passes.
- `just test` passes.
