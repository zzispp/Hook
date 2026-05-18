# Provider Key Endpoint Scope

## Goal

Align Hook provider API key endpoint support with Aether: a provider key can only declare `api_formats` that are already bound on the same provider's endpoints. Candidate routing then pairs provider endpoints and keys by endpoint `api_format`, allowing the existing format conversion path to decide whether the client format can be served through that provider endpoint.

## Scope

- Backend provider key create/update validation.
- Frontend provider key dialog supported-format options.
- Candidate routing verification against Aether semantics.
- Focused automated validation.

## Out of Scope

- Aether hub/proxy tunnels.
- Account pool behavior.
- New compatibility fallback for stale key formats.

## Acceptance

- Provider API key create/update rejects formats not present on the provider's endpoints.
- Provider key dialog only offers endpoint formats bound to the selected provider.
- Hook candidate routing continues to pair endpoint and key by provider endpoint format.
- Tests/checks run with explicit results recorded in `PROGRESS.md`.
