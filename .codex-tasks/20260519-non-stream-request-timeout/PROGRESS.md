# Progress

## 2026-05-19

- Confirmed no production DB work is needed.
- Confirmed `request_timeout_seconds` already exists in provider schema/type/candidate chain.
- Confirmed aether wraps upstream request execution with a per-request timeout.
- Implemented explicit non-stream total timeout usage for upstream send and full response body read.
- `cargo test -p backend timeout --no-default-features` passed.
- `cargo test -p backend non_stream --no-default-features` passed.
- `just check` passed.
- `just test` passed.
