# Progress

## Current state
- Real-flow harness is complete and passed against the local backend plus real Hook.rs/Ekan8 upstreams.
- The harness seeds local PostgreSQL fixtures, inserts the requested `menu_sections`, starts the local backend when needed, creates a proxy token, calls real upstreams through Hook backend, and verifies request records from the database.

## Notes
- Provider keys stay in process environment variables only.
- The script target is request boundary verification, not the full historical routing matrix.
- `node --check .codex-tasks/20260515-req-real-upstream/req_real_upstream_flow.mjs` passed.
- `REQ_REAL_HOOK_KEY=... REQ_REAL_EKAN8_KEY=... node .codex-tasks/20260515-req-real-upstream/req_real_upstream_flow.mjs` passed on 2026-05-15 10:06:50 +0800.
- Passing scenarios: admin upstream model fetch via `req`, Hook OpenAI non-stream, Hook OpenAI stream, Hook Claude, Ekan8 Gemini direct, Ekan8 OpenAI-compatible mapped model.
- Result evidence is stored in `.codex-tasks/20260515-req-real-upstream/raw/results.json`.
