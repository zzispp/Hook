# Remove Model Capability Gate

## Goal

Image generation routing should be decided by provider endpoint/key API format compatibility, not by the global model `supported_capabilities` metadata.

## Scope

- Update backend candidate matching so global model capabilities do not filter provider candidates.
- Keep endpoint and provider key `api_format` checks as the routing source of truth.
- Update focused backend tests.
- Run targeted tests and formatting.
