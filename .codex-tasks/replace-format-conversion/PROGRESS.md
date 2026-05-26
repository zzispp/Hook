# Progress

## 2026-05-26

- Confirmed `crates/formats` exists and old `crates/proxy/src/format_conversion` still exposes normalizer-backed conversion.
- Started replacement task as a Full Single task.
- Identified `formats::convert_request`, `formats::convert_response`, and `formats::StreamingStandardFormatMatrix` as the replacement surfaces.
- Replaced `proxy::format_conversion` registry and stream state with a `formats`-backed facade; `cargo check -p proxy` passes.
- Updated focused stream assertions to the `formats` crate stream emitter behavior; `cargo test -p proxy format_conversion_stream` passes.
- Deleted the old normalizer/schema helper modules from `proxy::format_conversion`; only the `formats` facade modules remain.
- Added explicit OpenAI Responses cross-format boundaries for unsupported request/response/stream item types.
- Validated `formats`, `proxy`, `backend`, and workspace checks; `cargo test -p proxy`, `cargo check -p backend`, and `just check` pass.
