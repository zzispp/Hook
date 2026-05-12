# Token usage metrics

## Goal

API token management should show real request count, last-used time, and spent quota for tokens used through the working proxy request path.

## Scope

- Inspect the proxy request flow and current token metric persistence.
- Add token metric updates at the real successful request boundary.
- Validate that request count, last-used time, and cost are updated from real persisted request data.
- Keep failures visible and avoid fake/mock success paths.
