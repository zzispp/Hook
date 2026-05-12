# Request Provider Flags

## Goal

Align the request records provider column with Aether: show the provider key name under the provider name and display a failover icon when a request executed more than one provider candidate, or a retry icon when an executed attempt has `retry_index > 0`.

## Scope

- Add request-record aggregate fields needed by the admin table.
- Keep semantics derived from `request_candidates`; do not add new runtime fallback behavior.
- Render icons in the provider column with backend i18n tooltips.

