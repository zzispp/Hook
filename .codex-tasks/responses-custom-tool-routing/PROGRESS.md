# Progress

## 2026-06-12

- Root cause confirmed from production request records: Responses `custom_tool_call` payloads fail only when routed to cross-format endpoints.
- Started a focused backend fix to filter incompatible conversion candidates before attempt execution.
- Added request feature detection for Responses custom tool input items.
- Candidate matching now excludes conversion endpoints when that feature is present, while preserving same-format `openai:cli` routing.
- Validation passed: `cargo fmt --check`, `timeout 60 cargo test -p hook_backend responses_custom_tool`, `timeout 60 cargo test -p hook_backend llm_proxy::candidate::selection::tests::matching`, and `timeout 60 cargo check -p hook_backend`.
