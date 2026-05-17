# Provider Cooldown Real Flow

Run a real local DB/backend/Redis integration flow for provider cooldown.

Requirements:
- Use real local Postgres and Redis.
- Use real upstream provider keys only through environment variables.
- Configure a deterministic 404 cooldown rule.
- Trigger cooldown through an upstream HTTP status response.
- Verify DB cooldown record, Redis cooldown key, scheduling skip, and manual release.
- Record machine-readable results in `raw/results.json`.

Validation:
- `node .codex-tasks/20260515-provider-cooldown-real-flow/provider_cooldown_real_flow.mjs`

