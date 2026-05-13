# Request Record Filter All Options

## Goal

Fix the request records toolbar so the "all status", "all format", and "all type" options can be selected reliably, matching the working behavior already used by the model and provider filters.

## Scope

- Request records toolbar filter state.
- Request records query filter serialization.
- Frontend validation for the request records admin page.

## Acceptance

- Selecting "all status", "all format", and "all type" keeps the control in a valid selected state.
- Clearing any request records filter sends no corresponding query filter to the API.
- Frontend lint/build checks pass.
