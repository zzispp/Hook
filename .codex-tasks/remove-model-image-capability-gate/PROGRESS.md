# Progress

## 2026-06-18

- Started from production diagnosis: `/v1/images/generations` currently fails because `gpt-image-2` lacks `image_generation` in global model `supported_capabilities`.
- User clarified desired behavior: route by API format only.
- Found existing coverage in `apps/hook_backend/src/llm_proxy/candidate/selection/tests/matching.rs` that asserts the old global model capability gate.
- Replaced old assertion with `matching_candidate_parts_routes_image_endpoint_without_global_capability`; it fails before production change with 0 candidates instead of 2.
- Removed the global model capability gate and the now-unused `CandidateRequest.required_capability` field. The single behavior test now passes.
- Ran `timeout 60 cargo test -p hook_backend matching_candidate_parts`: 28 tests passed.
- Ran `cargo fmt --all`, `just test`, and `timeout 60 cargo clippy -p hook_backend --all-targets -- -D warnings`; all passed.
