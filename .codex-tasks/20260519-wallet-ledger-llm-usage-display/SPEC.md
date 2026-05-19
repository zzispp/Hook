# Wallet Ledger LLM Usage Display

## Goal

Wallet ledger details must show user-facing LLM usage information instead of raw internal JSON snapshots.

## Scope

- Translate `llm_model_usage` as a wallet reason label.
- Translate `llm_request_record` as a wallet link type label.
- Replace LLM wallet settlement descriptions with concise text containing model, cost, API endpoint, and user token only.
- Keep provider, upstream key, pricing internals, user IDs, and wallet internals out of the ledger description.

## Validation

- Add or update backend tests before changing production behavior.
- Run focused backend tests.
- Run frontend lint/build checks when feasible for changed TypeScript.
