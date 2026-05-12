# Progress

## 2026-05-12

- Compared Hook with Aether. Aether keeps manual refresh loading separate from background polling, deduplicates full refreshes with an in-flight promise, polls active requests by status or pending billing, and merges incremental fields onto the existing row.
- Found Hook's two issues: auto refresh passes SWR `isValidating` to both refresh buttons, and stream proxy records do not set `first_byte_time_ms` until after the full upstream body is read.
- Updated request records UI so manual refresh has its own loading state. Auto refresh and active polling now call a deduplicated background refresh and no longer make the two refresh buttons spin.
- Updated active request handling so polling continues for `billing_status = pending` records, and row merges preserve already-known provider/key/format/timing/token fields when an active response omits them.
- Updated stream proxy recording so a successful upstream stream response first writes `streaming` with `first_byte_time_ms`, then finalizes the same attempt with `success`, usage, total latency, and the same first-byte value.
- Validation passed: `cargo test -p storage --test provider_request_records -- --nocapture`, `cargo check -p backend`, targeted frontend ESLint, `pnpm lint:frontend`, and `pnpm build:frontend`. The build exited successfully while printing an existing `Axios error: unauthorized` during static generation.
