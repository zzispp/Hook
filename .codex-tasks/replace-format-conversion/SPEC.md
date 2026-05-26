# Replace Format Conversion

## Goal

Replace Hook's legacy `proxy::format_conversion` normalizer implementation with the copied `formats` crate, while preserving the public integration surface needed by the backend.

## Boundaries

- Keep existing `ApiFormat`-based call sites compiling.
- Do not keep the legacy normalizer as a runtime fallback.
- Surface unsupported conversions as explicit errors.
- Validate with Rust checks and focused conversion tests.
