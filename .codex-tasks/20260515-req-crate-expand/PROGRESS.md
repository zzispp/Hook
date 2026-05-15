# Progress

## Current state
- Done. `crates/req` now owns generic HTTP request execution, response body/text/stream reading, shared header/status/url types, and websocket connection helpers.
- `provider` and `llm_proxy` now build and execute upstream HTTP/WebSocket requests through `req`; direct `reqwest` and `tokio-tungstenite` dependencies were removed from those crates.

## Notes
- The target is reuse of generic request/response plumbing, not moving the proxy's domain-specific workflow out of `llm_proxy`.

## Validation
- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test -p req`
- `just test`
