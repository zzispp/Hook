# Progress

## 2026-05-18

- Aether evidence: direct candidate building iterates active provider endpoints and active keys, then skips keys whose `api_formats` do not contain the endpoint `api_format`.
- Hook evidence: `matching.rs` and `proxy_candidate.rs` already filter key/endpoint pairs by provider endpoint `api_format`; format conversion is determined separately from client format to endpoint format.
- Current gap: backend key create/update validates only non-empty API formats, and frontend dialog uses global `API_FORMAT_OPTIONS`.
- Implemented backend validation: provider key create/update now requires selected `api_formats` to exist on the same provider's endpoints.
- Implemented storage cleanup: endpoint update/delete prunes removed endpoint formats from provider keys.
- Implemented model-fetch alignment: upstream model fetching now only tries key/endpoint pairs where the key supports the endpoint format.
- Implemented frontend alignment: provider key dialog lists formats from provider endpoints, shows stale selections as errors, and blocks save until selections are bound.
- Validation passed:
  - `cargo test -p provider key_endpoint_scope -- --nocapture`
  - `cargo test -p storage provider_endpoint_query -- --nocapture`
  - `cargo check`
  - `pnpm lint:frontend`
  - `pnpm build:frontend`
