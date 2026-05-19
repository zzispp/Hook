# SPEC

## Goal

Align Hook cache affinity with Aether's core behavior for provider routing:

- store complete affinity identity instead of only provider key id;
- promote a healthy cached provider/endpoint/key candidate explicitly;
- invalidate matching affinity on upstream failures;
- expose cached candidate state in request candidate records;
- keep retry expansion reserved for cached candidates.

## Constraints

- Do not add silent compatibility fallbacks for the old Redis value shape.
- Do not hide Redis or serialization errors.
- Keep behavior scoped to LLM proxy scheduling and request records.
- Validate with focused Rust tests and existing timeout rules.

## Evidence

- Hook currently writes Redis affinity as `{token_id}:{model_id}:{api_format} -> key_id`.
- Aether stores provider, endpoint, key, API format, model, timestamps, and request count.
- Aether invalidates matching affinity on retriable/non-client failures.
- Aether only expands provider retry slots for cached candidates.
