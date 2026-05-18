# Progress

## 2026-05-18

- Started by locating the current Hook provider model row placeholder and preparing to compare against aether's model test flow.
- Confirmed aether direct model test uses `provider.endpoints` as the endpoint source, fixes to `endpoint_id` when supplied, and filters active provider keys by the endpoint `api_format`.
- Confirmed aether model test executes candidates through `TaskService.execute_sync_candidates`, while the actual upstream check is adapter `check_endpoint`, which converts the OpenAI-style test body to the endpoint format and applies endpoint `body_rules` / `header_rules`.
- Adjusted Hook provider model test key selection to ignore key model whitelist for direct tests, match only the selected provider endpoint `api_format`, and try every eligible key until a success or accumulated failure.
- Validation passed: `cargo check -p provider -p types -p storage -p backend`, `cargo test -p provider eligible_endpoint_keys_does_not_require_model_binding`, and `pnpm --filter hook_frontend lint`.
- New scope at 17:11: user asked to route provider model tests through the unified candidate executor. Re-inspected Hook: the real executor is `llm_proxy::proxy::{prepare_proxy_request, execute_proxy_request}` in `apps/hook_backend`, while the current admin test path still calls `crates/provider` direct tester.
- Moved admin provider model tests into `apps/hook_backend/src/llm_proxy/model_test` behind a `ProviderModelTester` port injected into the provider API state.
- The test now builds a fixed `CandidateSelection` for the selected provider endpoint/model, then calls `proxy_fixed_json`, which records candidates, applies request conversion/body rules/header rules, executes retries through `execute_proxy_request`, and reads the recorded provider attempt back into the admin response.
- Preserved endpoint-specific stream semantics: `openai_compact` uses the same force-non-stream behavior as the real compact endpoint, while OpenAI Chat/CLI, Claude, and Gemini use the request stream flag.
- Removed the old provider-crate direct model tester surface and kept the provider crate decoupled from the proxy crate.
- Validation passed at 18:03: `cargo check -p provider -p types -p storage -p backend`; `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p provider`; `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy`; `rustfmt --edition 2024 --check apps/hook_backend/src/llm_proxy/model_test/mod.rs apps/hook_backend/src/llm_proxy/model_test/selection.rs apps/hook_backend/src/llm_proxy/model_test/candidate.rs apps/hook_backend/src/llm_proxy/model_test/response.rs apps/hook_backend/src/llm_proxy/proxy/request.rs apps/hook_backend/src/llm_proxy/proxy/executor.rs apps/hook_backend/src/llm_proxy/proxy/outbound_request.rs apps/hook_backend/src/llm_proxy/billing.rs`; `pnpm --filter hook_frontend lint`.
