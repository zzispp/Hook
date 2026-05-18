# Provider Model Test

## Goal

Add a real admin-side provider model test flow for the existing provider model row play action.

## Scope

- Replace the current unavailable toast with a model test dialog.
- Support selectable endpoints for the current implemented formats only: three OpenAI variants, one Claude variant, and one Gemini variant.
- Add backend request handling for the test call without mock success or silent fallback behavior.
- Route the test call through the same backend LLM proxy candidate execution path used by real requests.
- Keep failures visible in the response and UI.

## Validation

- Rust checks for touched backend crates.
- Frontend lint or targeted type/build validation where feasible.
