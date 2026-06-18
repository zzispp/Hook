# Progress

## 2026-06-18

- Production DB evidence: request `019eda05-3695-7030-9771-c7c883f11ac1` has `stream_end_reason=eof` and candidate `finished_at`, but both request and candidate statuses remain `streaming`.
- Prior failed request `019ed9f0-19f2-74f2-a109-04e6ad49271e` followed the same stream path and was later marked `stale_streaming_request` by stale sweep.
- Root cause: `prefetch()` can finish and record a terminal success before `stream_response()` calls `record_streaming_started()`. The later streaming-start patch overwrote the terminal `success` status back to `streaming`.
- Fix: skip `record_streaming_started()` when the relay is already finished or already recorded a terminal state.
- Validation passed: `cargo fmt --all`, `timeout 60 cargo test -p hook_backend image_stream`, `timeout 60 cargo test -p hook_backend stream_transport`, and `timeout 60 cargo clippy -p hook_backend --all-targets -- -D warnings`.
- Additional root cause from Aether comparison: Hook classified client `stream:true` image requests as streaming, but `openai_image` metadata had `stream_in_body=false`, so upstream image requests were rewritten as non-streaming while still using stream first-byte/watchdog timing.
- Planned second fix: preserve `stream:true` in OpenAI image generation/edit upstream bodies, matching Aether's OpenAI image request body construction.
- Implemented second fix: `openai_image` and `openai_image_edit` metadata now keep `stream` in the upstream request body while still avoiding `stream_options.include_usage`.
- Added regression tests for image endpoint metadata, image stream request rewrite, and force-non-stream rewrite.
- Validation passed after full fix: `cargo fmt --all`, `timeout 60 cargo test -p hook_backend openai_image`, `timeout 60 cargo test -p hook_backend stream_transport`, `timeout 60 cargo clippy -p hook_backend --all-targets -- -D warnings`, and `git diff --check`.
