# Request Record Active Refresh

## Goal

Make the request records page refresh like aether: normal records refresh handles list/page/filter freshness, while active-record polling keeps in-flight rows progressing from pending to streaming to terminal states in real time.

## Scope

- Request records list frontend polling.
- Active request records backend query semantics.
- Detail drawer freshness for selected records.

## Acceptance

- Active polling is independent of the global auto-refresh switch.
- Polling a known active request id can return its latest terminal state after completion.
- The selected detail drawer uses the latest merged record from the list when available.
- Backend and frontend checks pass.
