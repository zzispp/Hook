# Progress Log

## Session Start

- **Date**: 2026-05-16
- **Task name**: `20260516-stream-record-disconnect`
- **Task dir**: `.codex-tasks/20260516-stream-record-disconnect/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust / Axum / cargo tests

## Context Recovery Block

- **Current milestone**: complete
- **Current status**: DONE
- **Last completed**: #5 — Run final validation
- **Current artifact**: `TODO.csv`
- **Key context**: User reports streamed requests appear as one pending request record and frequent `stream disconnected before completion: Transport error: network error: error decoding response body` through local Hook backend at `127.0.0.1:5555`.
- **Known issues**: `just` binary is not installed in this shell; equivalent cargo commands were run with the repository 60-second timeout pattern.
- **Next action**: none.

---

## 2026-05-16 Validation Summary

- Root cause for missing billing: Hook forwarded OpenAI Chat streamed requests without `stream_options.include_usage=true`, so the upstream did not emit usage.
- Root cause for frequent ~30 second stream disconnects: Hook used `stream_first_byte_timeout_seconds` as the reqwest total request timeout for streams; the provider config has `stream_first_byte_timeout_seconds=30` and `request_timeout_seconds=300`.
- Additional stream robustness fix: usage parsing now handles SSE `data:` lines split across network chunks.
- Restarted backend on `127.0.0.1:5555` with the new code.
- Real curl validation succeeded for request `019e2f6b-92d3-7440-a357-462979c4c4e7`: `success/settled`, `prompt_tokens=27`, `completion_tokens=1403`, `total_tokens=1430`, `total_cost=0.04222500`.
- The recorded provider request body includes `stream_options.include_usage=true`.
- Wallet transaction table remained empty because the tested token is `independent`; token usage was updated to `used_quota=0.04222500`, `request_count=1`.
- `cargo fmt --all --check` passed.
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy:: -- --nocapture` passed with 44 tests.
- `perl ... 60 cargo test --workspace --lib --bins --tests` passed.
