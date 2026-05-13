# 03 Endpoint Resolution

Move endpoint choice from candidate construction into route execution.

Acceptance:
- Exact-format endpoint attempts happen before conversion endpoint attempts.
- Conversion fallback is explicit in trace metadata.
- Unsupported conversion still surfaces as a real error.
