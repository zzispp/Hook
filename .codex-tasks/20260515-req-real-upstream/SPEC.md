# Spec

Run a real local end-to-end validation for the extracted `crates/req` request boundary.

## Scope
- Seed dedicated real providers, endpoints, API keys, model bindings, billing group, and an API token in the local PostgreSQL database.
- Insert the required `menu_sections` rows supplied by the user.
- Use real upstreams:
  - Hook.rs at `https://www.hook.rs`
  - Ekan8 at `https://www.ekan8.com`
- Verify direct Hook.rs OpenAI-compatible non-stream and stream calls through Hook backend.
- Verify Ekan8 Gemini direct call through Hook backend.
- Verify Ekan8 OpenAI-compatible mapped model call through Hook backend.
- Verify admin upstream model fetch goes through the shared `req` crate path.
- Validate DB request records and request candidates for each real request.

## Safety
- Do not write upstream API keys or generated bearer tokens into task files.
- Read provider keys from environment variables at runtime.
- Use deterministic fixture IDs and mark/deactivate test rows during cleanup.
