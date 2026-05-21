# Format Conversion Refactor

## Goal

Refactor chat format conversion so OpenAI, Claude, and Gemini request/response/tool/thinking structures can convert between each other through one explicit internal model, using the three CLI projects under `/Users/bubu/Downloads/cli` as reference material.

## Boundaries

- Focus on Rust conversion code in `crates/proxy/src/format_conversion`.
- Keep behavior explicit: unsupported payload shapes must fail with concrete errors.
- Do not add mock success paths, silent fallback conversions, or compatibility shims.
- Preserve non-chat formats unless directly needed by conversion routing.

## Validation

- Add or update focused Rust tests that cover multi-direction conversion.
- Run targeted `cargo test -p proxy` tests first.
- Run broader Rust checks when feasible under the repository timeout policy.
